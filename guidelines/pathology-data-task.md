---
name: pathology-data-task
description: Curate digital pathology and whole slide image (WSI) datasets — tissue QC, gigapixel tile extraction, stain normalization, annotation QC, and multi-site harmonization for computational pathology.
recommended_skills:
  - histolab
  - pathml
  - monai
  - pydicom
  - albumentations
  - dataset-versioning
recommended_guidelines:
  - medical-bio-data-task
  - medical-imaging-task
  - cv-dataset-task
  - training-data-design-principles
---

## Overview

Whole slide images (WSI) are the largest files in medicine — a single slide can be 100K × 100K pixels, 20-50 GB uncompressed. Curation here isn't just image QC; it's about handling multi-resolution pyramids, stain variability across labs, gigapixel annotation, and the fact that pathologists disagree ~10-20% of the time even on cancer diagnosis.

## Phase 1: WSI File Format and Integrity

### 1a. WSI Formats

| Format | Typical Source | Tool |
|-------|-------|-------|
| `.svs` | Aperio/Leica | `openslide` / `tifffile` |
| `.ndpi` | Hamamatsu | `openslide` |
| `.scn` | Leica SCN | `openslide` |
| `.mrxs` | 3DHistech/MIRAX | `openslide` |
| `.vsi` | Olympus | `openslide` |
| `.tiff` (generic pyramidal) | Various | `tifffile` / `openslide` |
| DICOM WSI | Standardized | `pydicom` + `wsidicom` |

### 1b. WSI Integrity Checks

```python
import openslide

def validate_wsi(path):
    """Validate a WSI file for common issues."""
    slide = openslide.OpenSlide(path)
    issues = []
    
    # Check properties
    if "openslide.vendor" not in slide.properties:
        issues.append("unknown_vendor")
    
    # Level/dimension consistency
    dims = slide.dimensions
    if dims[0] < 100 or dims[1] < 100:
        issues.append("too_small")
    
    # Level count
    n_levels = slide.level_count
    if n_levels < 2:
        issues.append("single_level_only")
    
    # Magnification
    mag = slide.properties.get("openslide.objective-power", "unknown")
    
    # MPP (microns per pixel) — critical for analysis
    mpp_x = float(slide.properties.get("openslide.mpp-x", 0))
    if mpp_x <= 0:
        issues.append("missing_mpp")
    
    # Check each level
    for i in range(n_levels):
        try:
            slide.read_region((0, 0), i, slide.level_dimensions[i])
        except Exception:
            issues.append(f"level_{i}_unreadable")
    
    slide.close()
    return {
        "dimensions": dims,
        "n_levels": n_levels,
        "magnification": mag,
        "mpp_x": mpp_x,
        "issues": issues,
        "is_valid": len(issues) == 0 or all(
            i not in ("level_read_failed",) for i in issues
        ),
    }
```

### 1c. Common WSI Artifacts

| Artifact | Visual Appearance | Action |
|-------|-------|-------|
| **Tissue folds** | Dark lines or wrinkles | Flag region; exclude from analysis |
| **Air bubbles** | Round clear regions | Flag and exclude |
| **Pen marks** | Blue/black writing on tissue | Flag; exclude if over diagnostic area |
| **Out-of-focus regions** | Blurred tiles | Blur detection; exclude tiles |
| **Uneven staining** | Light/dark patches | Flag slide for re-staining review |
| **Scanning artifacts** | Stripes, stitching seams | Flag slide; re-scan if possible |

## Phase 2: Stain Normalization and Color QC

### 2a. Stain Variability

H&E staining varies across labs, days, and even within a slide. This is the #1 silent failure mode in computational pathology.

- **Stain normalization methods**: Macenko, Reinhard, Vahadane, StainGAN.
- **Normalize AFTER splitting** into train/val/test — normalize test to match train statistics, not vice versa.
- **Check stain matrix**: Extract hematoxylin and eosin vectors. Flag if the angle between H and E vectors is < 30° (poor separation).

```python
import numpy as np

def stain_separation_quality(rgb_patch):
    """Check that H&E stains separate well via color deconvolution."""
    # Simplified Macenko method check
    optical_density = -np.log(rgb_patch.clip(1e-6) / 255.0)
    od_flat = optical_density.reshape(-1, 3)
    
    # Check that colors form a plane (low singular value)
    _, s, _ = np.linalg.svd(od_flat - od_flat.mean(axis=0), full_matrices=False)
    singular_ratio = s[2] / s[0] if s[0] > 0 else 1.0
    # High ratio = colors don't form a plane = poor H&E separation
    return singular_ratio < 0.1  # True = good separation
```

### 2b. Color QC

- **Tissue vs. background**: > 30% of slide area should contain tissue.
- **Color saturation**: Flag slides with mean saturation > 2 SD from cohort mean.
- **White balance**: Check that background (no-tissue) regions are near-white in all channels.

## Phase 3: Annotation and Label QC

### 3a. Annotation Types

| Annotation | Format | Accuracy Requirement |
|-------|-------|-------|
| **Slide-level label** | String (cancer type, grade) | Must match pathology report |
| **ROI bounding box** | XYWH polygon | Within 100 μm of true boundary |
| **Pixel-level mask** | Binary or multi-class mask | Expert inter-observer Dice > 0.7 |
| **Cell/nucleus detection** | Centroid points | F1 > 0.9 for nuclei count |
| **Tumor grade** | WHO/ISUP grade | > 70% inter-pathologist agreement |

### 3b. Annotation QC Process

1. **Primary annotation** by a pathologist or trained annotator.
2. **Review by second pathologist** on a random stratified sample (10-20%).
3. **Adjudication**: a third pathologist resolves disagreements.
4. **Report inter-observer agreement**: Cohen's kappa for classification, Dice for segmentation.
5. **Flag low-confidence regions** where agreement is systematically poor.

### 3c. Common Label Errors

- **Edge effects**: Annotations extending to tissue edges where diagnosis is uncertain.
- **Ignore worst regions**: Annotators naturally skip the hardest cases. Audit what's NOT annotated.
- **Label leakage from report**: Using report keywords as labels without visual confirmation.
- **Temporal mismatch**: Biopsy date vs. annotation date — treatment may have occurred between.

## Phase 4: Tile Extraction and Dataset Assembly

### 4a. Tile Extraction Strategy

```python
import openslide
from pathlib import Path

def extract_tiles(slide_path, output_dir, tile_size=256, 
                   magnification=20, overlap=0, tissue_threshold=0.3):
    slide = openslide.OpenSlide(slide_path)
    
    # Find the level closest to target magnification
    target_mpp = 0.5  # ~20x
    best_level = 0
    for i in range(slide.level_count):
        mpp = float(slide.properties.get(f"openslide.level[{i}].mpp-x", 999))
        if abs(mpp - target_mpp) < abs(
            float(slide.properties.get(f"openslide.level[{best_level}].mpp-x", 999)) - target_mpp
        ):
            best_level = i
    
    w, h = slide.level_dimensions[best_level]
    # Extract tiles containing > tissue_threshold fraction of tissue
    for y in range(0, h - tile_size, tile_size - overlap):
        for x in range(0, w - tile_size, tile_size - overlap):
            tile = slide.read_region((x, y), best_level, (tile_size, tile_size))
            # Tissue detection (simplified)
            gray = np.array(tile.convert("L"))
            tissue_frac = (gray < 220).mean()
            if tissue_frac >= tissue_threshold:
                tile.save(f"{output_dir}/{x}_{y}.png")
    
    slide.close()
```

### 4b. Tile-Level QC

- **Blur detection**: Laplacian variance. Remove tiles below threshold.
- **Empty/background**: Remove tiles with < 10% tissue.
- **Pen/artifact detection**: Remove tiles with saturated blue/black regions.
- **Focus stacking**: For z-stack slides, pick the best-focused focal plane per tile.

## Phase 5: Multi-Site Harmonization

Computational pathology models are notoriously brittle across sites.

- **Stain normalization must be site-aware**: normalize each site independently.
- **Color augmentation at training time**: HED perturbation improves generalization.
- **External validation set from a DIFFERENT site is mandatory** — no model should ship without it.
- **Site confounders**: If all cancer cases come from site A and all normals from site B, the model learns the site, not the disease.

## Quality Gate

A pathology dataset is ready when:
- All WSIs pass integrity checks (readable, has MPP, multi-level pyramid).
- Tissue folds, bubbles, and pen marks are flagged or excluded.
- Stain normalization is applied and documented (method, target template).
- Inter-observer annotation agreement is measured and reported.
- Tile extraction parameters (size, magnification, overlap) are documented.
- Train/val/test splits are at the PATIENT level (not tile level).
- External validation set is from a different site.

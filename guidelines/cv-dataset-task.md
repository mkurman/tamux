---
name: cv-dataset-task
description: Curate image and video datasets for computer vision tasks — classification, detection, segmentation, multimodal. Covers augmentation strategy, annotation quality control, class balance, resolution requirements, synthetic data generation, and 2025-2026 techniques.
recommended_skills:
  - albumentations
  - monai
  - hf-datasets
  - embedding-analysis
  - dataset-versioning
recommended_guidelines:
  - training-data-design-principles
  - dataset-creation-curation-task
  - medical-imaging-task
---

## Overview

CV datasets live at the intersection of pixel quality, annotation accuracy, and augmentation strategy. Unlike text, images carry metadata (EXIF, resolution, color space) that silently corrupts training if mishandled. This guideline covers dataset design for classification, object detection, segmentation, and multimodal (CLIP-style) tasks.

## Phase 1: Image Quality Assurance

### 1a. Technical Quality Filters

| Issue | Detection | Action |
|-------|-------|-------|
| Corrupted files | `PIL.Image.verify()` or `imghdr` | Remove |
| Truncated images | File size vs. expected from dimensions | Remove |
| Duplicate images | Perceptual hash (pHash) or embedding cosine | Remove |
| Near-duplicates | Embedding similarity > 0.98 | Remove or flag |
| Low resolution | Min dimension threshold (e.g., < 224px) | Upsample or remove |
| Color space mismatch | Check mode (RGB vs. RGBA vs. L) | Convert to RGB |
| EXIF rotation | Read orientation tag, apply rotation | Normalize |
| Watermarks / logos | Classifier or template matching | Flag or remove |

Use `hf-datasets` with `.map()` for batch image validation.

### 1b. Dataset-Level Quality Metrics

```python
# Image statistics for normalization
from datasets import load_dataset
import numpy as np

dataset = load_dataset("my-image-dataset", split="train")

# Sample channel-wise mean/std
means, stds = [], []
for batch in dataset.iter(batch_size=256):
    pixels = np.array([np.array(img) for img in batch["image"]])
    means.append(pixels.mean(axis=(0, 1, 2)))
    stds.append(pixels.std(axis=(0, 1, 2)))

channel_mean = np.mean(means, axis=0)
channel_std = np.mean(stds, axis=0)
```

### 1c. Content Quality Filters

- **Blur detection**: Laplacian variance. Remove heavy motion blur.
- **Lighting**: Histogram analysis for over/underexposure.
- **Composition**: Optional — classifier for "aesthetic quality" for generation tasks.
- **NSFW/unsafe**: Classifier-based filter. Tune for your deployment context.

## Phase 2: Annotation Quality

### 2a. Classification Labels

- **Inter-annotator agreement**: Fleiss' kappa or Cohen's kappa. Target > 0.7.
- **Label noise detection**: Train a quick model, flag examples with high loss or inconsistent predictions across epochs (confidence-based cleaning).
- **Class balance**: Document distribution. For severe imbalance (< 1:20 ratio), use oversampling, synthetic generation, or weighted loss.

### 2b. Object Detection Annotations

- **Bounding box validation**:
  - No negative width/height.
  - Boxes within image boundaries.
  - No zero-area boxes.
  - Overlap check: duplicate boxes (IoU > 0.95) on same class.
- **Annotation format**: COCO JSON, YOLO txt, or Pascal VOC XML. Normalize to one format early.
- **Missing annotations audit**: Use an object detector to find high-confidence detections not in ground truth.

### 2c. Segmentation Masks

- **Mask integrity**: No holes where they shouldn't exist, pixel values in expected range.
- **Boundary quality**: Jaccard index between annotators on boundary pixels.
- **Class consistency**: Same object class gets same mask color/index across all images.
- **RLE encoding**: For large datasets, use COCO RLE or PNG compression.

### 2d. Annotation Tooling

| Tool | Best For | Scale |
|-------|-------|-------|
| Label Studio | General purpose, self-hosted | Small-medium |
| CVAT | Detection, segmentation, tracking | Medium-large |
| Labelbox | Enterprise, active learning | Large |
| Supervisely | Medical, satellite, specialized | Medium |
| VGG Image Annotator | Quick, lightweight | Small |

## Phase 3: Augmentation Strategy

Augmentation is NOT a post-processing step — it's part of the dataset design. Design augmentations BEFORE training.

### 3a. Augmentation by Task

| Task | Recommended Augmentations | Avoid |
|-------|-------|-------|
| **Classification** | Flip, rotation, color jitter, RandAugment | Heavy spatial transforms |
| **Detection** | Flip, crop, mosaic, mixup | Rotation unless boxes rotate too |
| **Segmentation** | Flip, rotation, elastic deform, grid distortion | Anything that doesn't transform masks |
| **Fine-grained** | Color jitter, slight rotation | Aggressive cropping |
| **Medical** | Elastic deform, intensity shift, gamma | Unrealistic color transforms |

Use `albumentations` for general CV, `monai` for medical imaging.

### 3b. Augmentation Validation

```python
import albumentations as A

transform = A.Compose([
    A.RandomResizedCrop(224, 224, scale=(0.8, 1.0)),
    A.HorizontalFlip(p=0.5),
    A.ColorJitter(brightness=0.2, contrast=0.2, saturation=0.2, hue=0.1, p=0.5),
    A.Normalize(mean=channel_mean, std=channel_std),
])

# Validate: apply to 100 images, visually inspect
# Check: no black images, no unrealistic colors, labels still visible
```

### 3c. Test-Time Augmentation (TTA)

Define TTA strategy at dataset design time, not at evaluation time. Document which transforms are TTA-safe (flip, multi-crop) vs. not (color jitter for medical).

## Phase 4: Multimodal (CLIP-Style) Datasets

### 4a. Image-Text Pair Quality

- **Alignment**: Does the caption describe the image? Use CLIP similarity or a VLM to score.
- **Specificity**: "A dog on a green lawn with a red ball" beats "A dog".
- **Noise**: Remove captions that are machine-generated gibberish, SEO spam, or alt-text cruft.
- **Language**: Filter to target language(s). Use `langdetect` on captions.

### 4b. Multimodal Filtering Pipeline

```
Raw pairs → Exact image dedup (pHash) → Text dedup (MinHash)
         → CLIP alignment score → Filter alignment < threshold
         → Caption quality (LLM judge) → Filter low-quality captions
         → Final dataset
```

## Phase 5: Synthetic Data

### 5a. When to Use Synthetic Images

- Class imbalance where real data is scarce.
- Privacy-sensitive domains (faces, medical).
- Rare scenarios (accidents, anomalies, edge cases).
- Controlled evaluation (lighting, pose, background).

### 5b. Generation Methods

| Method | Best For | Fidelity |
|-------|-------|-------|
| Stable Diffusion / Flux | General scenes, objects | High |
| ControlNet | Specific poses, layouts | High |
| 3D rendering (Blender, Unreal) | Precise control, multi-view | Very High |
| GANs (StyleGAN) | Faces, specific domains | High |
| Copy-paste augmentation | Detection data | N/A (uses real objects) |

**Critical**: Always flag synthetic images with `synthetic: true` in metadata. Never mix synthetic and real without the flag — it makes debugging impossible.

## Quality Gate

A CV dataset is ready when:
- All images pass technical quality filters (corruption, format, resolution).
- Annotations pass validation (boundary, format, inter-annotator agreement).
- Class distribution is documented and imbalance strategy is defined.
- Augmentation pipeline is designed, validated, and versioned.
- Train/val/test splits are created BEFORE augmentation (no leakage).
- Multimodal pairs pass alignment and caption quality checks.
- Synthetic data is flagged and documented.

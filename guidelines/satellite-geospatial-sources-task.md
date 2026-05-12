---
name: satellite-geospatial-sources-task
description: Find satellite and geospatial datasets for ML — Sentinel, Landsat, aerial imagery, SAR, LiDAR, building footprints, land cover, and disaster response.
recommended_skills:
  - geopandas
  - geomaster
  - database-lookup
recommended_guidelines:
  - cv-dataset-task
  - scientific-database-lookup-task
---

## Overview

Satellite and geospatial ML has exploded with foundation models (Prithvi, SatMAE, GFM). The data comes from public satellite constellations, aerial surveys, and curated benchmarks for specific tasks.

## Satellite Constellations (Free, Continuous)

| Constellation | Resolution | Revisit | Bands | Access |
|------|-------|-------|-------|-------|
| **Sentinel-2** (ESA) | 10m (RGB+NIR) | 5 days | 13 spectral | Copernicus Data Space / AWS |
| **Sentinel-1** (ESA) | 20m (SAR) | 6-12 days | C-band | Copernicus / AWS |
| **Landsat 8/9** (NASA/USGS) | 30m (15m pan) | 16 days | 11 spectral | USGS EarthExplorer / AWS |
| **MODIS** (NASA) | 250m-1km | Daily | 36 spectral | LP DAAC |
| **PlanetScope** (Planet) | 3m | Daily | 4-8 spectral | Commercial (free for research) |
| **GOES** (NOAA) | 500m-2km | 10 min | 16 spectral | AWS |

## Benchmark Datasets

| Dataset | Task | Size |
|------|-------|-------|
| **BigEarthNet** | Multi-label land cover, Sentinel-2 | 590K patches, 43 classes |
| **EuroSAT** | Land cover classification, Sentinel-2 | 27K images, 10 classes |
| **fMoW (Functional Map of the World)** | Temporal land use | 1M images, 62 categories |
| **SpaceNet** (1-8) | Building footprint, road extraction | VHR satellite, 11M+ buildings |
| **xView / xView2** | Object detection, disaster damage assessment | 1M+ objects, 60 classes |
| **DeepGlobe** | Road, building, land cover segmentation | VHR satellite |
| **FAIR1M** | Fine-grained object detection in aerial | 1M+ instances, 37 categories |
| **DOTA** | Oriented object detection aerial | 2,806 images, 15 categories |
| **RESISC45** | Scene classification, 45 classes | 31,500 images |
| **UC Merced** | Land use classification | 2,100 images, 21 classes |
| **SEN12MS** | Multimodal (Sentinel-1 + Sentinel-2) | 180K patches |

## Foundation Model Datasets

| Dataset | Use | Size |
|------|-------|-------|
| **SSL4EO** | Self-supervised learning on EO | 1M Sentinel-1/2, 250K Landsat |
| **Satlas** (Allen AI) | Pretraining, multi-modal | 302M labels, 137 categories |
| **GeoPile** | Geospatial pretraining corpus | 1.5M images, 6 modalities |
| **MMEarth** | Multi-modal Earth observation | 1.2M locations, 8 modalities |

## Specialized

| Dataset | Domain | Size |
|------|-------|-------|
| **CropHarvest** | Crop type mapping, global | 90K samples |
| **FloodNet** | Flood detection | UAV + satellite |
| **xBD** | Building damage assessment | 850K buildings, 6 disaster types |
| **OpenBuildings v3** (Google) | Building footprints | 1.8B buildings worldwide |
| **Dynamic World** (Google/WRI) | Near-real-time land cover | Global, 10m, Sentinel-2 |
| **Global Canopy Height** (Meta/WRI) | Tree height, worldwide | 1m resolution |
| **OpenStreetMap (OSM)** | Vector features | Global, community-maintained |
| **Microsoft Building Footprints** | Building polygons | 1.3B buildings |

## 3D and LiDAR

| Dataset | Description |
|------|-------|
| **USGS 3DEP** | LiDAR point clouds, USA |
| **AHN** | LiDAR, Netherlands |
| **DALES** | Aerial LiDAR semantic segmentation |
| **ISPRS Benchmarks** | Urban classification, 3D reconstruction |

## Quick Access

```python
# Sentinel-2 via STAC
import pystac_client
catalog = pystac_client.Client.open("https://earth-search.aws.element84.com/v1")
search = catalog.search(collections=["sentinel-2-l2a"], bbox=[-122.5, 37.5, -122.3, 37.7])

# BigEarthNet via HF
from datasets import load_dataset
ds = load_dataset("mmearth/bigearthnet", "s1")
```

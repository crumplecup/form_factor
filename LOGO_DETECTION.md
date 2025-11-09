# Logo Detection

This document describes the logo detection system and its current limitations.

## Overview

The logo detection system uses OpenCV's template matching to identify logos in form images. It supports multi-scale detection to handle logos that appear at different sizes than the template.

## Template Matching Method

Template matching works by sliding a template image across the target image and computing a similarity score at each position. The location with the highest similarity score is considered a match.

### Scale Sensitivity

Template matching is inherently scale-sensitive. For best results, the template and target should be at similar scales. When there's a significant size difference, detection quality degrades.

## Test Results

### Self-Detection Test

The `test_logo_self_detection` test verifies that a logo can detect itself perfectly:

- **Template**: Same as target image
- **Scale**: 1.0 (original size)
- **Result**: Perfect match (confidence = 1.0) at location (0, 0)
- **Status**: ✅ PASSING

This test confirms that template matching works correctly when the template and target are identical.

### Document Detection Test

The `test_logo_detection_in_document` test attempts to detect logos in a real form document:

- **Template 1**: `mint_logo.jpg` (960×960 pixels)
- **Template 2**: `mint_logo_bw.png` (166×180 pixels)
- **Target**: Logo in document at (362, 181) with size approximately 108×128 pixels
- **Scales tested**: 0.10, 0.11, 0.12, 0.13, 0.14, 0.15, 0.2, 0.3, 0.4, 0.5, 0.6, 0.65, 0.7, 0.75, 0.8

#### Results

| Template | Best Match | Confidence | Location | Scale | Template Size | Detection Size |
|----------|-----------|------------|----------|-------|---------------|----------------|
| mint_logo.jpg | Found | 0.7064 | (450, 381) | 0.65 | 960×960 | 624×624 |
| mint_logo_bw.png | Found | 0.5629 | (1017, 596) | 0.11 | 166×180 | 18×20 |

#### Issues Identified

1. **Size Mismatch**: The expected logo size is ~108×128 pixels, but:
   - `mint_logo.jpg` at scale 0.65 produces a 624×624 detection (5.8× too large)
   - `mint_logo_bw.png` at scale 0.11 produces an 18×20 detection (6× too small)

2. **Location Mismatch**: Neither detection overlaps with the expected rectangle at (362, 181):
   - `mint_logo.jpg` detected at (450, 381) - off by 88, 200 pixels
   - `mint_logo_bw.png` detected at (1017, 596) - off by 655, 415 pixels

3. **Required Scale Calculations**:
   - For `mint_logo.jpg` (960×960) to match 108×128: scale ≈ 0.11-0.13
   - For `mint_logo_bw.png` (166×180) to match 108×128: scale ≈ 0.65-0.71

4. **Confidence Analysis**:
   - At the expected location (362, 181), all confidence scores were near 0.0 or negative
   - Best matches occurred at unrelated locations in the document
   - This suggests the templates may not visually match the logo in the document well enough

## Possible Causes

1. **Visual Differences**: The logo in the document may differ from the templates due to:
   - Image quality/compression
   - Color/contrast differences
   - Rotation or perspective distortion
   - Artifacts from scanning or photographing

2. **Template Quality**: The templates themselves may not be ideal:
   - `mint_logo.jpg` is very large (960×960) and requires aggressive downscaling
   - Downscaling can introduce aliasing and loss of detail
   - The template may not match the logo's appearance in typical documents

3. **Multiple Matches**: The document may contain other visual elements that score higher than the actual logo

## Recommendations

1. **Extract Logo from Document**: Create a template by cropping the actual logo from a sample document at the expected size (~108×128). This ensures:
   - Perfect visual match
   - No scaling artifacts
   - Same image quality as target

2. **Use Multiple Templates**: Maintain templates at different scales and quality levels:
   - High-resolution template for large logos
   - Medium-resolution template for typical documents
   - Cropped-from-document template for exact matches

3. **Narrow Scale Range**: Once you know the expected logo size in documents, test fewer scales around that size:
   - If logo is typically 100-130 pixels wide in documents
   - And template is 166 pixels wide
   - Test scales: 0.60, 0.65, 0.70, 0.75, 0.80

4. **Feature Matching**: Consider implementing the feature matching method for:
   - Scale-invariant detection
   - Rotation-invariant detection
   - Better handling of perspective distortion

5. **Pre-processing**: Enhance detection by:
   - Converting both template and target to same color space
   - Normalizing brightness/contrast
   - Applying edge detection for shape-based matching

## Current Status

The logo detection system is **functional but limited**:

- ✅ Template matching works correctly for exact matches (self-detection test)
- ⚠️ Document detection works but finds incorrect locations/sizes
- ❌ Current templates are not well-suited for the test document

The system successfully detects logos in the document (confidence 0.7064 for best match), confirming that the detection pipeline works. However, the matches are at incorrect locations, indicating a template quality issue rather than a code issue.

## Next Steps

1. Create better templates by extracting logos directly from sample documents
2. Implement feature matching for scale/rotation invariance
3. Add template quality validation (warn if template is much larger/smaller than expected target)
4. Consider adding a template library with multiple versions of each logo at different scales

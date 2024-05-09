struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return FullscreenVertexOutput(clip_position, uv);
}

@group(0) @binding(0) var screenTexture: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> details: Details;

struct Details {
    edge_threshold_min: f32,
    edge_threshold_max: f32,
    iterations: i32,
    subpixel_quality: f32,
}

fn QUALITY(q: i32) -> f32 {
    switch (q) {
        default:              { return 1.0; }
        case 5:               { return 1.5; }
        case 6, 7, 8, 9:      { return 2.0; }
        case 10:              { return 4.0; }
        case 11:              { return 8.0; }
    }
}

fn rgb2luma(rgb: vec3<f32>) -> f32 {
    return sqrt(dot(rgb, vec3<f32>(0.299, 0.587, 0.114)));
}

@fragment
fn fs_main(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(screenTexture));
    let inverseScreenSize = 1.0 / resolution.xy;
    let texCoord = in.position.xy * inverseScreenSize;

    let centerSample = textureSampleLevel(screenTexture, samp, texCoord, 0.0);
    let colorCenter = centerSample.rgb;

    let lumaCenter = rgb2luma(colorCenter);

    let lumaDown = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(0, -1)).rgb);
    let lumaUp = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(0, 1)).rgb);
    let lumaLeft = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(-1, 0)).rgb);
    let lumaRight = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(1, 0)).rgb);

    let lumaMin = min(lumaCenter, min(min(lumaDown, lumaUp), min(lumaLeft, lumaRight)));
    let lumaMax = max(lumaCenter, max(max(lumaDown, lumaUp), max(lumaLeft, lumaRight)));

    let lumaRange = lumaMax - lumaMin;

    if (lumaRange < max(details.edge_threshold_min, lumaMax * details.edge_threshold_max)) {
        return centerSample;
    }

    let lumaDownLeft  = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(-1, -1)).rgb);
    let lumaUpRight   = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(1, 1)).rgb);
    let lumaUpLeft    = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(-1, 1)).rgb);
    let lumaDownRight = rgb2luma(textureSampleLevel(screenTexture, samp, texCoord, 0.0, vec2<i32>(1, -1)).rgb);

    let lumaDownUp = lumaDown + lumaUp;
    let lumaLeftRight = lumaLeft + lumaRight;

    let lumaLeftCorners = lumaDownLeft + lumaUpLeft;
    let lumaDownCorners = lumaDownLeft + lumaDownRight;
    let lumaRightCorners = lumaDownRight + lumaUpRight;
    let lumaUpCorners = lumaUpRight + lumaUpLeft;

    let edgeHorizontal = abs(-2.0 * lumaLeft   + lumaLeftCorners)  + 
                         abs(-2.0 * lumaCenter + lumaDownUp) * 2.0 + 
                         abs(-2.0 * lumaRight  + lumaRightCorners);

    let edgeVertical =   abs(-2.0 * lumaUp     + lumaUpCorners)       + 
                         abs(-2.0 * lumaCenter + lumaLeftRight) * 2.0 + 
                         abs(-2.0 * lumaDown   + lumaDownCorners);

    let isHorizontal = (edgeHorizontal >= edgeVertical);

    var stepLength = select(inverseScreenSize.x, inverseScreenSize.y, isHorizontal);

    var luma1 = select(lumaLeft, lumaDown, isHorizontal);
    var luma2 = select(lumaRight, lumaUp, isHorizontal);

    let gradient1 = luma1 - lumaCenter;
    let gradient2 = luma2 - lumaCenter;

    let is1Steepest = abs(gradient1) >= abs(gradient2);

    let gradientScaled = 0.25 * max(abs(gradient1), abs(gradient2));

    var lumaLocalAverage = 0.0;
    if (is1Steepest) {
        stepLength = -stepLength;
        lumaLocalAverage = 0.5 * (luma1 + lumaCenter);
    } else {
        lumaLocalAverage = 0.5 * (luma2 + lumaCenter);
    }

    var currentUv = texCoord;
    var offset = vec2<f32>(0.0, 0.0);
    if (isHorizontal) {
        currentUv.y = currentUv.y + stepLength * 0.5;
        offset.x = inverseScreenSize.x;
    } else {
        currentUv.x = currentUv.x + stepLength * 0.5;
        offset.y = inverseScreenSize.y;
    }

    var uv1 = currentUv - offset;
    var uv2 = currentUv + offset;

    var lumaEnd1 = rgb2luma(textureSampleLevel(screenTexture, samp, uv1, 0.0).rgb);
    var lumaEnd2 = rgb2luma(textureSampleLevel(screenTexture, samp, uv2, 0.0).rgb);
    lumaEnd1 = lumaEnd1 - lumaLocalAverage;
    lumaEnd2 = lumaEnd2 - lumaLocalAverage;

    var reached1 = abs(lumaEnd1) >= gradientScaled;
    var reached2 = abs(lumaEnd2) >= gradientScaled;
    var reachedBoth = reached1 && reached2;

    uv1 = select(uv1 - offset, uv1, reached1);
    uv2 = select(uv2 - offset, uv2, reached2);

    if (!reachedBoth) {
        for (var i: i32 = 2; i < details.iterations; i = i + 1) {
            if (!reached1) { 
                lumaEnd1 = rgb2luma(textureSampleLevel(screenTexture, samp, uv1, 0.0).rgb);
                lumaEnd1 = lumaEnd1 - lumaLocalAverage;
            }

            if (!reached2) { 
                lumaEnd2 = rgb2luma(textureSampleLevel(screenTexture, samp, uv2, 0.0).rgb);
                lumaEnd2 = lumaEnd2 - lumaLocalAverage;
            }

            reached1 = abs(lumaEnd1) >= gradientScaled;
            reached2 = abs(lumaEnd2) >= gradientScaled;
            reachedBoth = reached1 && reached2;

            if (!reached1) {
                uv1 = uv1 - offset * QUALITY(i);
            }
            if (!reached2) {
                uv2 = uv2 + offset * QUALITY(i);
            }

            if (reachedBoth) { 
                break; 
            }
        }
    }

    var distance1 = select(texCoord.y - uv1.y, texCoord.x - uv1.x, isHorizontal);
    var distance2 = select(uv2.y - texCoord.y, uv2.x - texCoord.x, isHorizontal);

    let isDirection1 = distance1 < distance2;
    let distanceFinal = min(distance1, distance2);

    let edgeThickness = (distance1 + distance2);

    let isLumaCenterSmaller = lumaCenter < lumaLocalAverage;

    let correctVariation1 = (lumaEnd1 < 0.0) != isLumaCenterSmaller;
    let correctVariation2 = (lumaEnd2 < 0.0) != isLumaCenterSmaller;

    var correctVariation = select(correctVariation2, correctVariation1, isDirection1);

    let pixelOffset = - distanceFinal / edgeThickness + 0.5;

    var finalOffset = select(0.0, pixelOffset, correctVariation);

    let lumaAverage = (1.0 / 12.0) * (2.0 * (lumaDownUp + lumaLeftRight) + lumaLeftCorners + lumaRightCorners);

    let subPixelOffset1 = clamp(abs(lumaAverage - lumaCenter) / lumaRange, 0.0, 1.0);
    let subPixelOffset2 = (-2.0 * subPixelOffset1 + 3.0) * subPixelOffset1 * subPixelOffset1;
    let subPixelOffsetFinal = subPixelOffset2 * subPixelOffset2 * details.subpixel_quality;

    finalOffset = max(finalOffset, subPixelOffsetFinal);

    var finalUv = texCoord;

    if (isHorizontal) {
        finalUv.y = finalUv.y + finalOffset * stepLength;
    } else {
        finalUv.x = finalUv.x + finalOffset * stepLength;
    }

    var finalColor = textureSampleLevel(screenTexture, samp, finalUv, 0.0).rgb;
    return vec4<f32>(finalColor, centerSample.a);
}

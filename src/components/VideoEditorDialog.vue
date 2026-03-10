<template>
    <el-dialog
        v-model="visible"
        title="视频编辑"
        width="980px"
        top="4vh"
        :close-on-click-modal="false"
    >
        <div class="video-editor-layout">
            <div class="preview-panel">
                <div
                    class="preview-stage"
                    ref="previewStageRef"
                    :style="{ aspectRatio: stageAspectRatio }"
                >
                    <video
                        ref="videoRef"
                        class="preview-video"
                        :src="videoPreviewUrl"
                        controls
                        @loadedmetadata="handleVideoLoadedMetadata"
                    />

                    <div
                        v-if="enableCrop && hasVideoMetadata"
                        class="crop-rect"
                        :style="cropPreviewStyle"
                    ></div>

                    <div
                        v-if="enableWatermark && watermarkPreviewUrl && hasVideoMetadata"
                        class="watermark-rect"
                        :style="watermarkPreviewStyle"
                        @mousedown="startWatermarkDrag"
                    >
                        <img :src="watermarkPreviewUrl" alt="watermark" draggable="false" />
                        <div
                            class="watermark-resize-handle"
                            @mousedown.stop="startWatermarkResize"
                        ></div>
                    </div>
                </div>
                <div class="preview-tip">提示：可直接拖动水印，拖右下角圆点缩放。</div>
            </div>

            <div class="control-panel">
                <el-form label-position="top" size="small">
                    <el-form-item label="编辑视频">
                        <el-select
                            v-model="selectedVideoPath"
                            filterable
                            placeholder="请选择视频"
                            style="width: 100%"
                        >
                            <el-option
                                v-for="video in availableVideos"
                                :key="String(video.id || video.path || video.filename)"
                                :label="buildVideoLabel(video)"
                                :value="String(video.path || '')"
                            />
                        </el-select>
                    </el-form-item>

                    <el-form-item label="裁剪">
                        <el-switch v-model="enableCrop" />
                    </el-form-item>
                    <div v-if="enableCrop" class="crop-controls">
                        <div class="slider-label-row">
                            <span>左侧裁剪 {{ cropLeftPct }}%</span>
                        </div>
                        <el-slider
                            :model-value="cropLeftPct"
                            :min="0"
                            :max="95 - cropRightPct"
                            @update:model-value="onCropLeftChange"
                        />
                        <div class="slider-label-row">
                            <span>右侧裁剪 {{ cropRightPct }}%</span>
                        </div>
                        <el-slider
                            :model-value="cropRightPct"
                            :min="0"
                            :max="95 - cropLeftPct"
                            @update:model-value="onCropRightChange"
                        />
                        <div class="slider-label-row">
                            <span>上侧裁剪 {{ cropTopPct }}%</span>
                        </div>
                        <el-slider
                            :model-value="cropTopPct"
                            :min="0"
                            :max="95 - cropBottomPct"
                            @update:model-value="onCropTopChange"
                        />
                        <div class="slider-label-row">
                            <span>下侧裁剪 {{ cropBottomPct }}%</span>
                        </div>
                        <el-slider
                            :model-value="cropBottomPct"
                            :min="0"
                            :max="95 - cropTopPct"
                            @update:model-value="onCropBottomChange"
                        />
                    </div>

                    <el-divider />

                    <el-form-item label="图片水印">
                        <el-switch v-model="enableWatermark" />
                    </el-form-item>
                    <div v-if="enableWatermark">
                        <el-form-item label="水印图片">
                            <el-input v-model="watermarkImagePath" readonly>
                                <template #append>
                                    <el-button @click="chooseWatermarkImage">选择</el-button>
                                </template>
                            </el-input>
                        </el-form-item>

                        <el-form-item label="水印大小">
                            <el-slider
                                :model-value="watermarkSizePct"
                                :min="4"
                                :max="80"
                                @update:model-value="onWatermarkSizeChange"
                            />
                        </el-form-item>
                    </div>

                    <el-divider />

                    <el-form-item label="输出文件">
                        <el-input v-model="outputPath" placeholder="请选择输出路径">
                            <template #append>
                                <el-button @click="chooseOutputPath">选择</el-button>
                            </template>
                        </el-input>
                    </el-form-item>
                </el-form>
            </div>
        </div>

        <template #footer>
            <div class="dialog-footer">
                <el-button @click="visible = false">取消</el-button>
                <el-button type="primary" :loading="exporting" @click="exportVideo">导出</el-button>
            </div>
        </template>
    </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'

interface VideoItem {
    id?: string
    title?: string
    videoname?: string
    filename?: string
    path?: string
}

interface CropPayload {
    x: number
    y: number
    width: number
    height: number
}

interface WatermarkPayload {
    imagePath: string
    x: number
    y: number
    width: number
    height: number
}

interface VideoEditRequest {
    inputPath: string
    outputPath: string
    crop?: CropPayload | null
    watermark?: WatermarkPayload | null
}

const props = withDefaults(
    defineProps<{
        modelValue: boolean
        videos: VideoItem[]
    }>(),
    {
        videos: () => []
    }
)

const emit = defineEmits<{
    'update:modelValue': [value: boolean]
    'export-success': [payload: { outputPath: string; inputPath: string }]
}>()

const visible = computed({
    get: () => props.modelValue,
    set: (value: boolean) => emit('update:modelValue', value)
})

const selectedVideoPath = ref('')
const outputPath = ref('')
const exporting = ref(false)

const enableCrop = ref(false)
const cropLeftPct = ref(0)
const cropRightPct = ref(0)
const cropTopPct = ref(0)
const cropBottomPct = ref(0)

const enableWatermark = ref(false)
const watermarkImagePath = ref('')
const watermarkPreviewUrl = ref('')
const watermarkImageAspect = ref(1)
const watermarkSizePct = ref(20)
const watermarkRect = ref({
    x: 0.05,
    y: 0.05,
    width: 0.2,
    height: 0.2
})

const videoRef = ref<HTMLVideoElement | null>(null)
const previewStageRef = ref<HTMLDivElement | null>(null)
const videoMetadata = ref({
    width: 0,
    height: 0
})

const availableVideos = computed(() =>
    (props.videos || []).filter(video => typeof video.path === 'string' && video.path.trim() !== '')
)

const hasVideoMetadata = computed(
    () => videoMetadata.value.width > 0 && videoMetadata.value.height > 0
)

const stageAspectRatio = computed(() => {
    if (!hasVideoMetadata.value) {
        return '16 / 9'
    }
    return `${videoMetadata.value.width} / ${videoMetadata.value.height}`
})

const videoPreviewUrl = computed(() =>
    selectedVideoPath.value ? convertFileSrc(selectedVideoPath.value) : ''
)

const cropPreviewStyle = computed(() => ({
    left: `${cropLeftPct.value}%`,
    top: `${cropTopPct.value}%`,
    width: `${100 - cropLeftPct.value - cropRightPct.value}%`,
    height: `${100 - cropTopPct.value - cropBottomPct.value}%`
}))

const watermarkPreviewStyle = computed(() => ({
    left: `${watermarkRect.value.x * 100}%`,
    top: `${watermarkRect.value.y * 100}%`,
    width: `${watermarkRect.value.width * 100}%`,
    height: `${watermarkRect.value.height * 100}%`
}))

const clamp = (value: number, min: number, max: number) => {
    return Math.max(min, Math.min(max, value))
}

const normalizePath = (path: string) => path.trim().replace(/\//g, '\\').toLowerCase()

const getFileName = (path: string) => {
    const normalized = path.replace(/\\/g, '/')
    const parts = normalized.split('/')
    return parts[parts.length - 1] || path
}

const getPathWithoutExt = (path: string) => {
    const fileName = getFileName(path)
    const dotIndex = fileName.lastIndexOf('.')
    return dotIndex > 0 ? fileName.slice(0, dotIndex) : fileName
}

const getDirPath = (path: string) => {
    const slashIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
    if (slashIndex < 0) return ''
    return path.slice(0, slashIndex + 1)
}

const buildDefaultOutputPath = (inputPath: string) => {
    if (!inputPath) return ''
    return `${getDirPath(inputPath)}${getPathWithoutExt(inputPath)}_edited.mp4`
}

const buildVideoLabel = (video: VideoItem) => {
    const title = (video.title || video.videoname || video.filename || '').trim()
    if (title) return title
    return getFileName(String(video.path || ''))
}

const syncDefaultVideo = () => {
    if (availableVideos.value.length === 0) {
        selectedVideoPath.value = ''
        return
    }

    const hasSelected = availableVideos.value.some(
        video => normalizePath(String(video.path || '')) === normalizePath(selectedVideoPath.value)
    )
    if (!hasSelected) {
        selectedVideoPath.value = String(availableVideos.value[0].path || '')
    }
}

const syncDefaultOutputPath = () => {
    if (!selectedVideoPath.value) {
        outputPath.value = ''
        return
    }
    outputPath.value = buildDefaultOutputPath(selectedVideoPath.value)
}

const onCropLeftChange = (value: number) => {
    cropLeftPct.value = clamp(Math.round(value), 0, 95 - cropRightPct.value)
}

const onCropRightChange = (value: number) => {
    cropRightPct.value = clamp(Math.round(value), 0, 95 - cropLeftPct.value)
}

const onCropTopChange = (value: number) => {
    cropTopPct.value = clamp(Math.round(value), 0, 95 - cropBottomPct.value)
}

const onCropBottomChange = (value: number) => {
    cropBottomPct.value = clamp(Math.round(value), 0, 95 - cropTopPct.value)
}

const computeWatermarkHeightByWidth = (nextWidth: number) => {
    if (!hasVideoMetadata.value || watermarkImageAspect.value <= 0) {
        return nextWidth
    }
    return (
        (nextWidth * videoMetadata.value.width) /
        (watermarkImageAspect.value * videoMetadata.value.height)
    )
}

const applyWatermarkWidth = (nextWidth: number) => {
    let width = clamp(nextWidth, 0.04, 0.8)
    let height = computeWatermarkHeightByWidth(width)

    if (height > 0.8) {
        const scale = 0.8 / height
        width *= scale
        height *= scale
    }
    if (height < 0.04) {
        height = 0.04
    }

    watermarkRect.value.width = width
    watermarkRect.value.height = height
    watermarkRect.value.x = clamp(watermarkRect.value.x, 0, 1 - watermarkRect.value.width)
    watermarkRect.value.y = clamp(watermarkRect.value.y, 0, 1 - watermarkRect.value.height)
    watermarkSizePct.value = Math.round(width * 100)
}

const onWatermarkSizeChange = (value: number) => {
    applyWatermarkWidth(value / 100)
}

const chooseWatermarkImage = async () => {
    const selected = await open({
        multiple: false,
        filters: [
            {
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp']
            }
        ]
    })

    const filePath = Array.isArray(selected) ? selected[0] : selected
    if (!filePath || typeof filePath !== 'string') {
        return
    }

    watermarkImagePath.value = filePath
    watermarkPreviewUrl.value = convertFileSrc(filePath)
    enableWatermark.value = true

    const img = new Image()
    img.onload = () => {
        watermarkImageAspect.value = img.naturalWidth / Math.max(1, img.naturalHeight)
        watermarkRect.value.x = 0.05
        watermarkRect.value.y = 0.05
        applyWatermarkWidth(0.2)
    }
    img.onerror = () => {
        watermarkImageAspect.value = 1
        applyWatermarkWidth(0.2)
    }
    img.src = watermarkPreviewUrl.value
}

const chooseOutputPath = async () => {
    if (!selectedVideoPath.value) {
        ElMessage.warning('请先选择视频')
        return
    }

    const selected = await save({
        title: '保存编辑后视频',
        defaultPath: outputPath.value || buildDefaultOutputPath(selectedVideoPath.value),
        filters: [
            {
                name: 'Video',
                extensions: ['mp4']
            }
        ]
    })

    if (selected && typeof selected === 'string') {
        outputPath.value = selected
    }
}

const handleVideoLoadedMetadata = () => {
    if (!videoRef.value) return
    videoMetadata.value.width = videoRef.value.videoWidth || 0
    videoMetadata.value.height = videoRef.value.videoHeight || 0
    if (enableWatermark.value && watermarkImagePath.value) {
        applyWatermarkWidth(watermarkRect.value.width)
    }
}

type PointerMode = 'none' | 'drag' | 'resize'
const pointerState = {
    mode: 'none' as PointerMode,
    startX: 0,
    startY: 0,
    originX: 0,
    originY: 0,
    originWidth: 0
}

const clearPointerListeners = () => {
    window.removeEventListener('mousemove', handlePointerMove)
    window.removeEventListener('mouseup', stopPointerAction)
}

const stopPointerAction = () => {
    pointerState.mode = 'none'
    clearPointerListeners()
}

const startWatermarkDrag = (event: MouseEvent) => {
    if (!enableWatermark.value || !watermarkPreviewUrl.value) return
    pointerState.mode = 'drag'
    pointerState.startX = event.clientX
    pointerState.startY = event.clientY
    pointerState.originX = watermarkRect.value.x
    pointerState.originY = watermarkRect.value.y
    window.addEventListener('mousemove', handlePointerMove)
    window.addEventListener('mouseup', stopPointerAction)
}

const startWatermarkResize = (event: MouseEvent) => {
    if (!enableWatermark.value || !watermarkPreviewUrl.value) return
    pointerState.mode = 'resize'
    pointerState.startX = event.clientX
    pointerState.startY = event.clientY
    pointerState.originWidth = watermarkRect.value.width
    window.addEventListener('mousemove', handlePointerMove)
    window.addEventListener('mouseup', stopPointerAction)
}

const handlePointerMove = (event: MouseEvent) => {
    const stage = previewStageRef.value
    if (!stage) return

    const rect = stage.getBoundingClientRect()
    if (rect.width <= 0 || rect.height <= 0) return

    const dx = event.clientX - pointerState.startX
    const dy = event.clientY - pointerState.startY

    if (pointerState.mode === 'drag') {
        const nextX = pointerState.originX + dx / rect.width
        const nextY = pointerState.originY + dy / rect.height
        watermarkRect.value.x = clamp(nextX, 0, 1 - watermarkRect.value.width)
        watermarkRect.value.y = clamp(nextY, 0, 1 - watermarkRect.value.height)
    } else if (pointerState.mode === 'resize') {
        const nextWidth = pointerState.originWidth + dx / rect.width
        applyWatermarkWidth(nextWidth)
    }
}

const buildCropPayload = (): CropPayload | null => {
    if (!enableCrop.value || !hasVideoMetadata.value) return null
    const widthScale = 1 - (cropLeftPct.value + cropRightPct.value) / 100
    const heightScale = 1 - (cropTopPct.value + cropBottomPct.value) / 100

    const cropX = Math.round((videoMetadata.value.width * cropLeftPct.value) / 100)
    const cropY = Math.round((videoMetadata.value.height * cropTopPct.value) / 100)
    const cropWidth = Math.round(videoMetadata.value.width * widthScale)
    const cropHeight = Math.round(videoMetadata.value.height * heightScale)

    if (cropWidth <= 0 || cropHeight <= 0) {
        return null
    }

    return {
        x: cropX,
        y: cropY,
        width: cropWidth,
        height: cropHeight
    }
}

const buildWatermarkPayload = (): WatermarkPayload | null => {
    if (!enableWatermark.value || !watermarkImagePath.value || !hasVideoMetadata.value) {
        return null
    }

    const width = Math.round(videoMetadata.value.width * watermarkRect.value.width)
    const height = Math.round(videoMetadata.value.height * watermarkRect.value.height)
    const x = Math.round(videoMetadata.value.width * watermarkRect.value.x)
    const y = Math.round(videoMetadata.value.height * watermarkRect.value.y)

    if (width <= 0 || height <= 0) {
        return null
    }

    return {
        imagePath: watermarkImagePath.value,
        x,
        y,
        width,
        height
    }
}

const exportVideo = async () => {
    if (!selectedVideoPath.value) {
        ElMessage.warning('请先选择视频')
        return
    }

    if (!outputPath.value.trim()) {
        ElMessage.warning('请先选择输出路径')
        return
    }

    const crop = buildCropPayload()
    const watermark = buildWatermarkPayload()
    if (!crop && !watermark) {
        ElMessage.warning('请至少开启裁剪或图片水印')
        return
    }

    const payload: VideoEditRequest = {
        inputPath: selectedVideoPath.value,
        outputPath: outputPath.value.trim(),
        crop,
        watermark
    }

    exporting.value = true
    try {
        const exportedPath = await invoke<string>('edit_video_with_watermark', { request: payload })
        ElMessage.success('视频导出完成')
        emit('export-success', {
            outputPath: exportedPath,
            inputPath: selectedVideoPath.value
        })
    } catch (error) {
        ElMessage.error(`视频导出失败: ${error}`)
    } finally {
        exporting.value = false
    }
}

const resetEditorState = () => {
    enableCrop.value = false
    cropLeftPct.value = 0
    cropRightPct.value = 0
    cropTopPct.value = 0
    cropBottomPct.value = 0

    enableWatermark.value = false
    watermarkImagePath.value = ''
    watermarkPreviewUrl.value = ''
    watermarkImageAspect.value = 1
    watermarkRect.value = {
        x: 0.05,
        y: 0.05,
        width: 0.2,
        height: 0.2
    }
    watermarkSizePct.value = 20
}

watch(
    () => visible.value,
    async (isVisible: boolean) => {
        if (!isVisible) {
            stopPointerAction()
            return
        }
        syncDefaultVideo()
        syncDefaultOutputPath()
        resetEditorState()
        await nextTick()
        if (videoRef.value) {
            videoRef.value.load()
        }
    }
)

watch(
    () => availableVideos.value,
    () => {
        syncDefaultVideo()
        if (visible.value) {
            syncDefaultOutputPath()
        }
    },
    { deep: true }
)

watch(
    () => selectedVideoPath.value,
    async () => {
        videoMetadata.value.width = 0
        videoMetadata.value.height = 0
        syncDefaultOutputPath()
        await nextTick()
        if (videoRef.value) {
            videoRef.value.load()
        }
    }
)

onBeforeUnmount(() => {
    stopPointerAction()
})
</script>

<style scoped>
.video-editor-layout {
    display: grid;
    grid-template-columns: 1.2fr 1fr;
    gap: 16px;
    min-height: 500px;
}

.preview-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.preview-stage {
    width: 100%;
    max-height: 66vh;
    background: #111;
    border-radius: 8px;
    overflow: hidden;
    position: relative;
    border: 1px solid #333;
}

.preview-video {
    width: 100%;
    height: 100%;
    display: block;
    object-fit: fill;
    background: #000;
}

.crop-rect {
    position: absolute;
    border: 2px dashed #ffd04b;
    background: rgba(255, 208, 75, 0.12);
    box-sizing: border-box;
    pointer-events: none;
}

.watermark-rect {
    position: absolute;
    border: 1px solid #67c23a;
    background: rgba(103, 194, 58, 0.12);
    box-sizing: border-box;
    cursor: move;
}

.watermark-rect img {
    width: 100%;
    height: 100%;
    display: block;
    user-select: none;
    pointer-events: none;
}

.watermark-resize-handle {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: #67c23a;
    border: 1px solid #fff;
    position: absolute;
    right: -6px;
    bottom: -6px;
    cursor: nwse-resize;
}

.control-panel {
    border: 1px solid #ebeef5;
    border-radius: 8px;
    padding: 12px;
    overflow: auto;
    max-height: 66vh;
}

.crop-controls {
    margin-top: -4px;
}

.slider-label-row {
    display: flex;
    justify-content: space-between;
    color: #606266;
    font-size: 12px;
}

.preview-tip {
    color: #909399;
    font-size: 12px;
}

.dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}

@media (max-width: 1200px) {
    .video-editor-layout {
        grid-template-columns: 1fr;
    }

    .control-panel {
        max-height: none;
    }
}
</style>

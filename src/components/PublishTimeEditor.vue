<template>
  <div class="publish-time-editor">
    <section
      class="publish-time-editor-panel publish-time-editor-panel--left-drop"
      :class="{ 'publish-time-drop-active': isLeftDropTargetActive }"
      data-drop-target="left"
      @dragenter.prevent.stop="allowDrop"
      @dragover.prevent.stop="allowDrop"
      @drop.prevent.stop="handleDropToLeft"
    >
      <div class="publish-time-editor-panel-header">
        <span>进行中视频</span>
        <span class="publish-time-editor-header-meta">
          {{ loading ? '加载中...' : `${runningCards.length} 个视频 / ${seasonCount} 个合集` }}
        </span>
      </div>
      <div v-if="loading || runningCards.length === 0" class="publish-time-editor-empty">
        <el-empty :description="loading ? '正在加载进行中视频...' : '暂无进行中视频'" :image-size="88" />
      </div>
      <div
        v-else
        class="publish-date-list publish-date-list--left-drop"
        @dragenter.prevent.stop="allowDrop"
        @dragover.prevent.stop="allowDrop"
        @drop.prevent.stop="handleDropToLeft"
      >
        <div
          v-for="seasonRow in seasonRows"
          :key="`season-${seasonRow.key}`"
          class="publish-date-row"
          :class="{
            'publish-date-row--cancelled': seasonRow.hasUnscheduled,
            'publish-time-drop-active': isLeftDropTargetActive
          }"
          @dragenter.prevent.stop="allowDrop"
          @dragover.prevent.stop="allowDrop"
          @drop.prevent.stop="handleDropToLeft"
        >
          <div
            class="publish-date-row-header publish-group-row-header publish-group-row-header--draggable"
            :class="{ 'publish-group-row-header--dragging': draggingSeasonKey === seasonRow.key }"
            draggable="true"
            @click="toggleSeasonGroup(seasonRow.key)"
            @dragstart.stop="handleSeasonDragStart($event, seasonRow.key)"
            @dragend.stop="handleSeasonDragEnd"
          >
            <span>{{ seasonRow.label }}</span>
            <div class="publish-date-row-header-right">
              <span class="publish-date-row-count">{{ seasonRow.tasks.length }}</span>
              <el-icon class="publish-group-arrow" :class="{ collapsed: isSeasonGroupCollapsed(seasonRow.key) }">
                <ArrowDown />
              </el-icon>
            </div>
          </div>
          <div
            v-if="seasonRow.tasks.length > 0 && !isSeasonGroupCollapsed(seasonRow.key)"
            class="publish-date-row-cards"
            @dragenter.prevent.stop="allowDrop"
            @dragover.prevent.stop="allowDrop"
            @drop.prevent.stop="handleDropToLeft"
          >
            <div
              v-for="task in seasonRow.tasks"
              :key="`running-${seasonRow.key}-${task.id}`"
              class="publish-time-video-card"
              :class="{
                'publish-time-video-card--cancelled': task.isUnscheduled,
                'publish-time-video-card--selected': isCardSelected(task.id),
                'publish-time-video-card--dragging': isCardDragging(task.id)
              }"
              draggable="false"
              @dragenter.prevent.stop="allowDrop"
              @dragover.prevent.stop="allowDrop"
              @drop.prevent.stop="handleDropToLeft"
              @mousedown.left.stop="handleCardPointerDown($event, task.id)"
            >
              <div class="publish-time-video-card-title-row">
                <div class="publish-time-video-card-title">{{ task.videoTitle }}</div>
                <div class="publish-time-video-card-season" :title="task.seasonTitle">{{ task.seasonTitle }}</div>
              </div>
              <div class="publish-time-video-card-meta">
                <span>{{ task.scheduledText }}</span>
                <span v-if="task.bvid">{{ task.bvid }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="publish-time-editor-panel">
      <div class="publish-time-editor-panel-header">
        <span>定时发布日期</span>
        <div class="publish-time-editor-header-actions">
          <div class="publish-date-range-editor" @click.stop>
            <span class="publish-date-range-label">提交间隔(s)</span>
            <el-input
              :model-value="String(submitIntervalSeconds)"
              size="small"
              class="publish-submit-interval-input"
              placeholder="1"
              @update:model-value="setSubmitIntervalSeconds($event)"
            />
          </div>
          <div class="publish-date-range-editor" @click.stop>
            <span class="publish-date-range-label">随机时间</span>
            <el-input
              :model-value="globalTimeRange"
              size="small"
              class="publish-date-range-input"
              placeholder="12:00-24:00"
              @update:model-value="setGlobalTimeRange($event)"
            />
          </div>
          <span class="publish-time-editor-header-meta">
            已设置 {{ scheduledVideoCount }}，未设置 {{ unscheduledRunningVideoCount }}
          </span>
        </div>
      </div>
      <div class="publish-date-list">
        <div
          v-for="dateRow in dateRows"
          :key="dateRow.key"
          class="publish-date-row"
          :class="{ 'publish-time-drop-active': isDateDropTargetActive(dateRow.key) }"
          :data-date-key="dateRow.key"
          data-drop-target="date"
          @dragenter.prevent.stop="allowDrop"
          @dragover.prevent.stop="allowDrop"
          @drop.prevent.stop="handleDropToDateGroup($event, dateRow.key)"
        >
          <div class="publish-date-row-header publish-group-row-header" @click="toggleDateGroup(dateRow.key)">
            <div class="publish-date-row-header-main">
              <span>{{ dateRow.label }}</span>
            </div>
            <div class="publish-date-row-header-right">
              <el-button
                v-if="dateRow.tasks.length > 0"
                size="small"
                text
                type="primary"
                class="publish-date-row-clear-btn"
                @click.stop="clearDateGroup(dateRow.key)"
              >
                清空
              </el-button>
              <span class="publish-date-row-count">{{ dateRow.tasks.length }}</span>
              <el-icon class="publish-group-arrow" :class="{ collapsed: isDateGroupCollapsed(dateRow.key) }">
                <ArrowDown />
              </el-icon>
            </div>
          </div>
          <div
            v-if="dateRow.tasks.length > 0 && !isDateGroupCollapsed(dateRow.key)"
            class="publish-date-row-cards"
            :data-date-key="dateRow.key"
            data-drop-target="date"
            @dragenter.prevent.stop="allowDrop"
            @dragover.prevent.stop="allowDrop"
            @drop.prevent.stop="handleDropToDateGroup($event, dateRow.key)"
          >
            <div
              v-for="task in dateRow.tasks"
              :key="`date-${dateRow.key}-${task.id}`"
              class="publish-time-video-card publish-time-video-card--compact"
              :class="{
                'publish-time-video-card--changed': task.isChanged && !!task.scheduleDateKey,
                'publish-time-video-card--selected': isCardSelected(task.id),
                'publish-time-video-card--dragging': isCardDragging(task.id)
              }"
              :data-date-key="dateRow.key"
              data-drop-target="date"
              draggable="false"
              @dragenter.prevent.stop="allowDrop"
              @dragover.prevent.stop="allowDrop"
              @drop.prevent.stop="handleDropToDateGroup($event, dateRow.key)"
              @mousedown.left.stop="handleCardPointerDown($event, task.id)"
            >
              <div class="publish-time-video-card-title-row">
                <div class="publish-time-video-card-title">{{ task.videoTitle }}</div>
                <div class="publish-time-video-card-season" :title="task.seasonTitle">{{ task.seasonTitle }}</div>
              </div>
              <div class="publish-time-video-card-meta">
                <span>{{ task.scheduledTimeOnly }}</span>
                <span v-if="task.bvid">{{ task.bvid }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div class="publish-time-submit-bar">
        <div class="publish-time-submit-meta">
          <span>待提交 {{ changedCards.length }}</span>
          <span v-if="submitProgress">提交中 {{ submitProgress.done }}/{{ submitProgress.total }}</span>
        </div>
        <el-button
          type="primary"
          :loading="submitting"
          :disabled="loading || !allVideosScheduled || changedCards.length === 0 || submitting"
          @click="submitChanges"
        >
          提交修改
        </el-button>
      </div>
    </section>
    <div
      v-if="customCardDrag.active"
      class="publish-time-drag-ghost"
      :style="{
        left: `${customCardDrag.x}px`,
        top: `${customCardDrag.y}px`
      }"
    >
      {{ customCardDrag.label }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowDown } from '@element-plus/icons-vue'
import { useUtilsStore } from '../stores/utils'
import { useUploadStore } from '../stores/upload'

const props = defineProps<{ selectedUser: any | null }>()

const utilsStore = useUtilsStore()
const uploadStore = useUploadStore()

type PublishTimeCard = {
  id: string
  aid: number
  bvid: string
  videoTitle: string
  seasonTitle: string
  originalDtime?: number
  originalDateKey: string
  assignedDtime?: number
  assignedDateKey: string
  orderIndex: number
}

type RunningCardView = PublishTimeCard & {
  scheduleDateKey: string
  dtime?: number
  scheduledText: string
  scheduledTimeOnly: string
  isUnscheduled: boolean
  isChanged: boolean
}

const DEFAULT_PUBLISH_TIME_RANGE = '12:00-24:00'
const PUBLISH_TIME_MAX_DAYS = 15

const loading = ref(false)
const seasonCount = ref(0)
const submitting = ref(false)
const submitProgress = ref<{ done: number; total: number } | null>(null)
const workingCards = ref<PublishTimeCard[]>([])
const seasonGroupCollapsed = ref<Record<string, boolean>>({})
const dateGroupCollapsed = ref<Record<string, boolean>>({})
const draggingCardId = ref('')
const draggingCardIds = ref<string[]>([])
const draggingSeasonKey = ref('')
const selectedCardIds = ref<string[]>([])
const selectionAnchorCardId = ref('')
const globalTimeRange = ref(DEFAULT_PUBLISH_TIME_RANGE)
const submitIntervalSeconds = ref(1)
const customCardDrag = ref({
  active: false,
  x: 0,
  y: 0,
  label: ''
})
const customCardDropTarget = ref<{ type: 'left' } | { type: 'date'; dateKey: string } | null>(null)
let loadToken = 0
let orderSeed = 1
let pendingCardDrag:
  | {
      startX: number
      startY: number
      payload: { type: 'card'; id: string } | { type: 'cards'; ids: string[] }
    }
  | null = null

const formatTwoDigits = (value: number) => String(value).padStart(2, '0')
const formatDateKey = (date: Date) =>
  `${date.getFullYear()}-${formatTwoDigits(date.getMonth() + 1)}-${formatTwoDigits(date.getDate())}`
const formatDateLabel = (date: Date) =>
  `${date.getMonth() + 1}/${date.getDate()} (${['周日', '周一', '周二', '周三', '周四', '周五', '周六'][date.getDay()]})`

const getLatestSchedulable = (baseDate: Date = new Date()) => {
  const latest = new Date(baseDate.getTime() + PUBLISH_TIME_MAX_DAYS * 24 * 60 * 60 * 1000)
  return {
    dtime: Math.floor(latest.getTime() / 1000),
    dateKey: formatDateKey(latest)
  }
}

const formatDateTime = (dtime?: number) => {
  if (!dtime) return '未设置'
  const date = new Date(dtime * 1000)
  return `${formatDateKey(date)} ${formatTwoDigits(date.getHours())}:${formatTwoDigits(date.getMinutes())}`
}

const formatTimeOnly = (dtime?: number) => {
  if (!dtime) return '--:--'
  const date = new Date(dtime * 1000)
  return `${formatTwoDigits(date.getHours())}:${formatTwoDigits(date.getMinutes())}`
}

const rebuildWorkingCards = (videos: any[]) => {
  orderSeed = 1
  const list = Array.isArray(videos) ? videos : []
  workingCards.value = list
    .filter((video: any) => video?.is_in_progress !== false)
    .map((video: any) => {
      const dtime = typeof video?.dtime === 'number' && video.dtime > 0 ? Number(video.dtime) : undefined
      const dateKey = dtime ? formatDateKey(new Date(dtime * 1000)) : ''
      return {
        id: String(video?.id || video?.aid || video?.bvid || ''),
        aid: Number(video?.aid || 0),
        bvid: String(video?.bvid || ''),
        videoTitle: String(video?.title || video?.bvid || '未命名视频'),
        seasonTitle: String(video?.season_title || '未加入合集'),
        originalDtime: dtime,
        originalDateKey: dateKey,
        assignedDtime: dtime,
        assignedDateKey: dateKey,
        orderIndex: orderSeed++
      } as PublishTimeCard
    })
    .sort((a, b) =>
      `${a.seasonTitle}|${a.videoTitle}|${a.bvid}|${a.aid}`.localeCompare(
        `${b.seasonTitle}|${b.videoTitle}|${b.bvid}|${b.aid}`,
        'zh-CN'
      )
    )
    .map(card => ({ ...card, orderIndex: orderSeed++ }))
}

const formatAssignedText = (card: PublishTimeCard) => {
  if (!card.assignedDateKey) return '未设置'
  if (!card.assignedDtime) return `${card.assignedDateKey} 随机时间`
  return formatDateTime(card.assignedDtime)
}

const formatAssignedTimeOnly = (card: PublishTimeCard) => {
  if (!card.assignedDateKey) return '--'
  if (!card.assignedDtime) return '随机'
  return formatTimeOnly(card.assignedDtime)
}

const runningCards = computed<RunningCardView[]>(() =>
  workingCards.value
    .slice()
    .sort((a, b) => a.orderIndex - b.orderIndex)
    .map(card => {
      const isUnscheduled = !card.assignedDateKey
      const isChanged = Boolean(
        card.originalDateKey !== card.assignedDateKey ||
          (card.originalDateKey &&
            card.assignedDateKey &&
            card.originalDtime !== card.assignedDtime)
      )
      return {
        ...card,
        scheduleDateKey: card.assignedDateKey,
        dtime: card.assignedDtime,
        scheduledText: formatAssignedText(card),
        scheduledTimeOnly: formatAssignedTimeOnly(card),
        isUnscheduled,
        isChanged
      }
    })
)

const seasonRows = computed(() => {
  const grouped = new Map<string, RunningCardView[]>()
  for (const task of runningCards.value) {
    const key = String(task.seasonTitle || '未加入合集')
    if (!grouped.has(key)) grouped.set(key, [])
    grouped.get(key)!.push(task)
  }
  return Array.from(grouped.entries())
    .map(([key, tasks]) => ({
      key,
      label: key,
      hasUnscheduled: tasks.some(task => task.isUnscheduled),
      tasks: tasks.slice().sort((a, b) => a.orderIndex - b.orderIndex)
    }))
    .sort((a, b) => a.label.localeCompare(b.label, 'zh-CN'))
})

const availableDateList = computed(() => {
  const now = new Date()
  const minTime = new Date(now.getTime() + 2 * 60 * 60 * 1000)
  const maxTime = new Date(now.getTime() + PUBLISH_TIME_MAX_DAYS * 24 * 60 * 60 * 1000)
  const cursor = new Date(minTime.getFullYear(), minTime.getMonth(), minTime.getDate())
  const end = new Date(maxTime.getFullYear(), maxTime.getMonth(), maxTime.getDate())
  const result: Array<{ key: string; label: string }> = []
  while (cursor.getTime() <= end.getTime()) {
    const d = new Date(cursor)
    result.push({ key: formatDateKey(d), label: formatDateLabel(d) })
    cursor.setDate(cursor.getDate() + 1)
  }
  return result
})

const dateRows = computed(() => {
  const grouped = new Map<string, RunningCardView[]>()
  for (const task of runningCards.value) {
    if (!task.scheduleDateKey) continue
    if (!grouped.has(task.scheduleDateKey)) grouped.set(task.scheduleDateKey, [])
    grouped.get(task.scheduleDateKey)!.push(task)
  }
  return availableDateList.value.map(item => ({
    ...item,
    tasks: (grouped.get(item.key) || []).slice().sort((a, b) => {
      const aTime = a.dtime || 0
      const bTime = b.dtime || 0
      return a.orderIndex - b.orderIndex || aTime - bTime || a.videoTitle.localeCompare(b.videoTitle, 'zh-CN')
    })
  }))
})

const scheduledVideoCount = computed(() => runningCards.value.filter(task => task.scheduleDateKey).length)
const unscheduledRunningVideoCount = computed(() => runningCards.value.filter(task => !task.scheduleDateKey).length)
const changedCards = computed(() => runningCards.value.filter(card => card.isChanged))
const allVideosScheduled = computed(
  () => runningCards.value.length > 0 && runningCards.value.every(card => Boolean(card.scheduleDateKey))
)

const isSeasonGroupCollapsed = (key: string) => Boolean(seasonGroupCollapsed.value[key])
const toggleSeasonGroup = (key: string) => {
  seasonGroupCollapsed.value[key] = !seasonGroupCollapsed.value[key]
}
const isDateGroupCollapsed = (key: string) => Boolean(dateGroupCollapsed.value[key])
const toggleDateGroup = (key: string) => {
  dateGroupCollapsed.value[key] = !dateGroupCollapsed.value[key]
}

const getOrderedCardIds = () => runningCards.value.map(card => card.id)

const setSelectedCardIds = (ids: string[]) => {
  const validIds = new Set(getOrderedCardIds())
  const uniqueIds: string[] = []
  for (const id of ids) {
    if (!id || !validIds.has(id) || uniqueIds.includes(id)) continue
    uniqueIds.push(id)
  }
  selectedCardIds.value = uniqueIds
}

const isCardSelected = (cardId: string) => selectedCardIds.value.includes(cardId)
const isCardDragging = (cardId: string) => draggingCardIds.value.includes(cardId)
const isLeftDropTargetActive = computed(
  () => customCardDrag.value.active && customCardDropTarget.value?.type === 'left'
)
const isDateDropTargetActive = (dateKey: string) =>
  customCardDrag.value.active &&
  customCardDropTarget.value?.type === 'date' &&
  customCardDropTarget.value.dateKey === dateKey

const handleCardMouseDown = (event: MouseEvent, cardId: string) => {
  if (event.button !== 0) return

  const orderedIds = getOrderedCardIds()
  if (!orderedIds.includes(cardId)) return

  const toggleKey = event.ctrlKey || event.metaKey

  if (event.shiftKey) {
    const anchorId =
      (selectionAnchorCardId.value && orderedIds.includes(selectionAnchorCardId.value)
        ? selectionAnchorCardId.value
        : selectedCardIds.value.find(id => orderedIds.includes(id))) || cardId
    const anchorIndex = orderedIds.indexOf(anchorId)
    const targetIndex = orderedIds.indexOf(cardId)
    const [start, end] = anchorIndex <= targetIndex ? [anchorIndex, targetIndex] : [targetIndex, anchorIndex]
    const rangeIds = orderedIds.slice(start, end + 1)
    setSelectedCardIds(toggleKey ? [...selectedCardIds.value, ...rangeIds] : rangeIds)
    return
  }

  if (toggleKey) {
    const wasSelected = isCardSelected(cardId)
    if (isCardSelected(cardId)) {
      selectedCardIds.value = selectedCardIds.value.filter(id => id !== cardId)
    } else {
      selectedCardIds.value = [...selectedCardIds.value, cardId]
    }
    if (!wasSelected) selectionAnchorCardId.value = cardId
    return
  }

  if (selectedCardIds.value.length > 1 && isCardSelected(cardId)) {
    selectionAnchorCardId.value = cardId
    return
  }

  setSelectedCardIds([cardId])
  selectionAnchorCardId.value = cardId
}

const getCardDragPayload = (
  cardId: string
): { type: 'card'; id: string } | { type: 'cards'; ids: string[] } => {
  const selectedIds = isCardSelected(cardId) ? selectedCardIds.value.slice() : [cardId]
  return selectedIds.length > 1 ? { type: 'cards', ids: selectedIds } : { type: 'card', id: cardId }
}

const getCardDragLabel = (payload: { type: 'card'; id: string } | { type: 'cards'; ids: string[] }) => {
  if (payload.type === 'cards') return `移动 ${payload.ids.length} 个视频`
  const card = runningCards.value.find(item => item.id === payload.id)
  return card?.videoTitle || '移动视频'
}

const setGlobalTimeRange = (value: string | null | undefined) => {
  globalTimeRange.value = String(value || '').trim()
}

const setSubmitIntervalSeconds = (value: string | number | null | undefined) => {
  const parsed = Number(value)
  if (!Number.isFinite(parsed)) {
    submitIntervalSeconds.value = 1
    return
  }
  submitIntervalSeconds.value = Math.min(60, Math.max(0, Math.round(parsed * 10) / 10))
}

const parseTimeRange = (rangeText: string) => {
  const normalized = String(rangeText || '')
    .trim()
    .replace(/[，：]/g, ':')
    .replace(/\s+/g, '')
  const match = normalized.match(/^(\d{1,2}):(\d{2})-(\d{1,2}):(\d{2})$/)
  if (!match) return null
  const sh = Number(match[1])
  const sm = Number(match[2])
  const eh = Number(match[3])
  const em = Number(match[4])
  const startValid = sh >= 0 && sh <= 23 && sm >= 0 && sm <= 59
  const endValid = (eh >= 0 && eh <= 23 && em >= 0 && em <= 59) || (eh === 24 && em === 0)
  if (!startValid || !endValid) return null
  const startSec = sh * 3600 + sm * 60
  const endSec = eh * 3600 + em * 60
  if (endSec <= startSec) return null
  return { startSec, endSec }
}

const randomInt = (min: number, max: number) => {
  if (max <= min) return min
  return Math.floor(Math.random() * (max - min + 1)) + min
}

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

const buildRandomDtimeForDateKey = (dateKey: string, rangeText: string) => {
  const range = parseTimeRange(rangeText) || parseTimeRange(DEFAULT_PUBLISH_TIME_RANGE)
  if (!range) return undefined
  const [y, m, d] = dateKey.split('-').map(Number)
  if (!y || !m || !d) return undefined
  const totalSeconds = randomInt(range.startSec, range.endSec - 1)
  const hh = Math.floor(totalSeconds / 3600)
  const mm = Math.floor((totalSeconds % 3600) / 60)
  const ss = randomInt(0, 59)
  const dt = new Date(y, m - 1, d, hh, mm, ss)
  return Math.floor(dt.getTime() / 1000)
}

const assignCardToDate = (cardId: string, dateKey: string) => {
  const card = workingCards.value.find(item => item.id === cardId)
  if (!card) return
  const latest = getLatestSchedulable()
  card.assignedDateKey = dateKey
  if (dateKey && dateKey === latest.dateKey) {
    card.assignedDtime = latest.dtime
  } else if (!card.assignedDtime || formatDateKey(new Date(card.assignedDtime * 1000)) !== dateKey) {
    card.assignedDtime = undefined
  }
  card.orderIndex = ++orderSeed
}

const clearCardSchedule = (cardId: string) => {
  const card = workingCards.value.find(item => item.id === cardId)
  if (!card) return
  card.assignedDateKey = ''
  card.assignedDtime = undefined
  card.orderIndex = ++orderSeed
}

const assignCardsToDate = (cardIds: string[], dateKey: string) => {
  for (const cardId of cardIds) assignCardToDate(cardId, dateKey)
}

const clearCardsSchedule = (cardIds: string[]) => {
  for (const cardId of cardIds) clearCardSchedule(cardId)
}

const assignSeasonToDate = (seasonKey: string, dateKey: string) => {
  if (!seasonKey || !dateKey) return
  const latest = getLatestSchedulable()
  for (const card of workingCards.value) {
    if (String(card.seasonTitle || '') !== seasonKey) continue
    if (card.assignedDateKey) continue
    card.assignedDateKey = dateKey
    if (dateKey === latest.dateKey) {
      card.assignedDtime = latest.dtime
    } else if (!card.assignedDtime || formatDateKey(new Date(card.assignedDtime * 1000)) !== dateKey) {
      card.assignedDtime = undefined
    }
    card.orderIndex = ++orderSeed
  }
}

const clearSeasonSchedule = (seasonKey: string) => {
  if (!seasonKey) return
  for (const card of workingCards.value) {
    if (String(card.seasonTitle || '') !== seasonKey) continue
    card.assignedDateKey = ''
    card.assignedDtime = undefined
    card.orderIndex = ++orderSeed
  }
}

const clearDateGroup = (dateKey: string) => {
  if (!dateKey) return
  for (const card of workingCards.value) {
    if (card.assignedDateKey !== dateKey) continue
    card.assignedDateKey = ''
    card.assignedDtime = undefined
    card.orderIndex = ++orderSeed
  }
}

const applyCardPayload = (
  payload: { type: 'card'; id: string } | { type: 'cards'; ids: string[] },
  target: { type: 'left' } | { type: 'date'; dateKey: string } | null
) => {
  if (!target) return
  if (target.type === 'left') {
    if (payload.type === 'cards') {
      clearCardsSchedule(payload.ids)
      return
    }
    clearCardSchedule(payload.id)
    return
  }

  if (payload.type === 'cards') {
    assignCardsToDate(payload.ids, target.dateKey)
    return
  }
  assignCardToDate(payload.id, target.dateKey)
}

const resolveCustomDropTarget = (clientX: number, clientY: number) => {
  const element = document.elementFromPoint(clientX, clientY) as HTMLElement | null
  const dropTargetElement = element?.closest?.('[data-drop-target]') as HTMLElement | null
  if (!dropTargetElement) return null
  const targetType = String(dropTargetElement.dataset.dropTarget || '')
  if (targetType === 'left') return { type: 'left' } as const
  if (targetType === 'date') {
    const dateKey = String(dropTargetElement.dataset.dateKey || '')
    if (dateKey) return { type: 'date', dateKey } as const
  }
  return null
}

const resetCustomCardDrag = () => {
  pendingCardDrag = null
  customCardDrag.value = {
    active: false,
    x: 0,
    y: 0,
    label: ''
  }
  customCardDropTarget.value = null
  draggingCardIds.value = []
  draggingCardId.value = ''
  document.body.style.cursor = ''
}

const handleGlobalPointerMove = (event: MouseEvent) => {
  if (!pendingCardDrag) return

  const offsetX = event.clientX - pendingCardDrag.startX
  const offsetY = event.clientY - pendingCardDrag.startY
  const distance = Math.hypot(offsetX, offsetY)

  if (!customCardDrag.value.active && distance < 6) {
    return
  }

  if (!customCardDrag.value.active) {
    customCardDrag.value = {
      active: true,
      x: event.clientX + 14,
      y: event.clientY + 14,
      label: getCardDragLabel(pendingCardDrag.payload)
    }
    draggingCardIds.value =
      pendingCardDrag.payload.type === 'cards'
        ? pendingCardDrag.payload.ids.slice()
        : [pendingCardDrag.payload.id]
    draggingCardId.value = pendingCardDrag.payload.type === 'card' ? pendingCardDrag.payload.id : ''
    draggingSeasonKey.value = ''
    document.body.style.cursor = 'grabbing'
  } else {
    customCardDrag.value = {
      ...customCardDrag.value,
      x: event.clientX + 14,
      y: event.clientY + 14
    }
  }

  customCardDropTarget.value = resolveCustomDropTarget(event.clientX, event.clientY)
  event.preventDefault()
}

const handleGlobalPointerUp = (event: MouseEvent) => {
  if (!pendingCardDrag) return
  const activePayload = pendingCardDrag.payload
  const hadActiveDrag = customCardDrag.value.active
  const target = hadActiveDrag ? resolveCustomDropTarget(event.clientX, event.clientY) : null
  resetCustomCardDrag()
  if (!hadActiveDrag) return
  applyCardPayload(activePayload, target)
}

const handleCardPointerDown = (event: MouseEvent, cardId: string) => {
  if (event.button !== 0) return
  handleCardMouseDown(event, cardId)
  pendingCardDrag = {
    startX: event.clientX,
    startY: event.clientY,
    payload: getCardDragPayload(cardId)
  }
}

const setTransparentDragImage = (event: DragEvent) => {
  if (!event.dataTransfer) return
  const canvas = document.createElement('canvas')
  canvas.width = 1
  canvas.height = 1
  event.dataTransfer.setDragImage(canvas, 0, 0)
}

const setDragTransferData = (dataTransfer: DataTransfer, payload: string) => {
  dataTransfer.effectAllowed = 'move'
  dataTransfer.dropEffect = 'move'
  dataTransfer.setData('application/x-biliup-publish-time-drag', payload)
  dataTransfer.setData('text/plain', payload)
}

const handleSeasonDragStart = (event: DragEvent, seasonKey: string) => {
  if (event.dataTransfer) {
    setDragTransferData(event.dataTransfer, `season:${seasonKey}`)
  }
  setTransparentDragImage(event)
  requestAnimationFrame(() => {
    draggingSeasonKey.value = seasonKey
    draggingCardId.value = ''
    draggingCardIds.value = []
  })
}

const handleSeasonDragEnd = () => {
  draggingSeasonKey.value = ''
  draggingCardIds.value = []
}

const getDragPayload = (
  event: DragEvent
): { type: 'card'; id: string } | { type: 'cards'; ids: string[] } | { type: 'season'; key: string } | null => {
  if (draggingCardIds.value.length > 1) return { type: 'cards', ids: draggingCardIds.value.slice() }
  if (draggingCardId.value) return { type: 'card', id: draggingCardId.value }
  if (draggingSeasonKey.value) return { type: 'season', key: draggingSeasonKey.value }
  const raw =
    event.dataTransfer?.getData('application/x-biliup-publish-time-drag') ||
    event.dataTransfer?.getData('text/plain') ||
    ''
  if (!raw) return null
  if (raw.startsWith('cards:')) {
    try {
      const ids = JSON.parse(raw.slice(6))
      if (Array.isArray(ids) && ids.length > 0) {
        return { type: 'cards', ids: ids.map(id => String(id)).filter(Boolean) }
      }
    } catch {
      return null
    }
  }
  if (raw.startsWith('card:')) return { type: 'card', id: raw.slice(5) }
  if (raw.startsWith('season:')) return { type: 'season', key: raw.slice(7) }
  return { type: 'card', id: raw }
}

const allowDrop = (event: DragEvent) => {
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
}

const handleDropToDateGroup = (event: DragEvent, dateKey: string) => {
  allowDrop(event)
  const payload = getDragPayload(event)
  if (!payload) return
  if (payload.type === 'cards') {
    assignCardsToDate(payload.ids, dateKey)
    return
  }
  if (payload.type === 'card') {
    assignCardToDate(payload.id, dateKey)
    return
  }
  assignSeasonToDate(payload.key, dateKey)
}

const handleDropToLeft = (event: DragEvent) => {
  allowDrop(event)
  const payload = getDragPayload(event)
  if (!payload) return
  if (payload.type === 'cards') {
    clearCardsSchedule(payload.ids)
    return
  }
  if (payload.type === 'card') {
    clearCardSchedule(payload.id)
    return
  }
  clearSeasonSchedule(payload.key)
}

const loadVideos = async () => {
  const token = ++loadToken
  const uid = Number(props.selectedUser?.uid || 0)
  if (!uid) {
    seasonCount.value = 0
    workingCards.value = []
    selectedCardIds.value = []
    selectionAnchorCardId.value = ''
    draggingCardIds.value = []
    draggingCardId.value = ''
    draggingSeasonKey.value = ''
    return
  }
  loading.value = true
  try {
    const result = await utilsStore.getWebArchivesPubingForPublishTime(uid)
    if (token !== loadToken) return
    const videos = Array.isArray((result as any)?.videos) ? (result as any).videos : []
    seasonCount.value = Number((result as any)?.season_count || 0)
    rebuildWorkingCards(videos)
    setSelectedCardIds(selectedCardIds.value)
  } catch (error) {
    if (token !== loadToken) return
    seasonCount.value = 0
    workingCards.value = []
    selectedCardIds.value = []
    selectionAnchorCardId.value = ''
    draggingCardIds.value = []
    draggingCardId.value = ''
    draggingSeasonKey.value = ''
    console.error('加载修改发布时间视频列表失败:', error)
    utilsStore.showMessage(`加载修改发布时间视频列表失败: ${error}`, 'error')
  } finally {
    if (token === loadToken) loading.value = false
  }
}

const submitChanges = async () => {
  if (submitting.value) return
  const uid = Number(props.selectedUser?.uid || 0)
  if (!uid) {
    utilsStore.showMessage('请先选择用户', 'warning')
    return
  }
  if (!allVideosScheduled.value) {
    utilsStore.showMessage('请先为全部进行中的视频设置发布时间后再提交', 'warning')
    return
  }
  const currentChangedCards = changedCards.value
  if (currentChangedCards.length === 0) {
    utilsStore.showMessage('没有需要提交的修改', 'info')
    return
  }
  submitting.value = true
  submitProgress.value = { done: 0, total: currentChangedCards.length }
  try {
    const latestSchedulable = getLatestSchedulable()
    const submitIntervalMs = Math.max(0, Math.round(Number(submitIntervalSeconds.value || 0) * 1000))
    for (let index = 0; index < currentChangedCards.length; index++) {
      const card = currentChangedCards[index]
      const videoId = card.bvid || String(card.aid || '')
      if (!videoId) throw new Error(`视频标识缺失: ${card.videoTitle}`)
      const detail: any = await utilsStore.getVideoDetail(uid, videoId)
      let nextDtime: number | undefined
      if (card.assignedDateKey) {
        if (card.assignedDateKey === latestSchedulable.dateKey) {
          nextDtime = latestSchedulable.dtime
        } else {
          nextDtime = buildRandomDtimeForDateKey(card.assignedDateKey, globalTimeRange.value || DEFAULT_PUBLISH_TIME_RANGE)
          if (!nextDtime) throw new Error(`随机时间范围格式无效: ${globalTimeRange.value}`)
        }
      }
      detail.dtime = nextDtime
      await uploadStore.submitTemplate(uid, detail)
      const target = workingCards.value.find(item => item.id === card.id)
      if (target) {
        target.assignedDtime = nextDtime
        target.originalDtime = nextDtime
        target.originalDateKey = target.assignedDateKey
      }
      submitProgress.value = { done: index + 1, total: currentChangedCards.length }
      if (submitIntervalMs > 0 && index < currentChangedCards.length - 1) {
        await sleep(submitIntervalMs)
      }
    }
    utilsStore.showMessage(`已提交 ${currentChangedCards.length} 个视频的发布时间修改`, 'success')
  } catch (error) {
    console.error('批量提交发布时间修改失败:', error)
    const remainingCount = changedCards.value.length
    utilsStore.showMessage(`批量提交失败: ${error}，可再次点击继续提交剩余 ${remainingCount} 项`, 'error')
  } finally {
    submitting.value = false
  }
}

watch(
  () => props.selectedUser?.uid,
  () => {
    seasonGroupCollapsed.value = {}
    dateGroupCollapsed.value = {}
    void loadVideos()
  },
  { immediate: true }
)

onMounted(() => {
  window.addEventListener('mousemove', handleGlobalPointerMove)
  window.addEventListener('mouseup', handleGlobalPointerUp)
  if (props.selectedUser?.uid) void loadVideos()
})

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleGlobalPointerMove)
  window.removeEventListener('mouseup', handleGlobalPointerUp)
  resetCustomCardDrag()
})
</script>

<style scoped>
.publish-time-editor {
  display: grid;
  grid-template-columns: minmax(320px, 1fr) minmax(380px, 1.25fr);
  gap: 12px;
  min-height: calc(100vh - 220px);
}

.publish-time-editor-panel {
  border: 1px solid #ebeef5;
  border-radius: 12px;
  background: #fff;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.publish-time-editor-panel--left-drop {
  min-height: 220px;
}

.publish-time-editor-panel-header {
  padding: 12px 14px;
  border-bottom: 1px solid #f0f2f5;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  font-size: 14px;
  font-weight: 700;
  color: #303133;
}

.publish-time-editor-header-actions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.publish-time-editor-header-meta {
  font-size: 12px;
  color: #909399;
  font-weight: 400;
}

.publish-time-editor-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 280px;
}

.publish-date-list {
  padding: 12px;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.publish-date-list--left-drop {
  min-height: 220px;
}

.publish-date-row {
  border: 1px solid #edf0f5;
  border-radius: 8px;
  overflow: hidden;
  background: #fff;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.publish-date-row--cancelled {
  border-color: #f5c9c9;
  background: #fff8f8;
}

.publish-date-row--cancelled .publish-date-row-header {
  background: #fff1f1;
}

.publish-date-row-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background: #f7f9fc;
  color: #303133;
  font-size: 13px;
  font-weight: 600;
}

.publish-date-row-header-main {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.publish-group-row-header {
  cursor: pointer;
  user-select: none;
}

.publish-group-row-header:hover {
  background: #eef4ff;
}

.publish-group-row-header--draggable {
  cursor: grab;
}

.publish-group-row-header--dragging {
  opacity: 0.75;
  cursor: grabbing;
  background: #e8f3ff;
}

.publish-date-row-header-right {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-left: 8px;
}

.publish-date-row-clear-btn {
  padding: 0 4px;
  color: #409eff;
}

.publish-date-row-clear-btn:hover {
  color: #66b1ff;
}

.publish-date-row-count {
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: #e6f0ff;
  color: #2f6fd6;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.publish-group-arrow {
  color: #909399;
  transition: transform 0.2s ease, color 0.2s ease;
}

.publish-group-row-header:hover .publish-group-arrow {
  color: #409eff;
}

.publish-group-arrow.collapsed {
  transform: rotate(-90deg);
}

.publish-date-row-cards {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.publish-time-video-card {
  border: 1px solid #ebeef5;
  border-radius: 8px;
  padding: 7px 10px;
  background: linear-gradient(180deg, #ffffff 0%, #fbfdff 100%);
  display: flex;
  flex-direction: column;
  gap: 4px;
  user-select: none;
  -webkit-user-select: none;
}

.publish-time-video-card[draggable='true'] {
  cursor: grab;
}

.publish-time-video-card--dragging {
  opacity: 0.55;
  cursor: grabbing;
}

.publish-time-video-card--compact {
  padding: 6px 9px;
  background: #fafcff;
}

.publish-time-video-card--cancelled {
  border-color: #f56c6c;
  background: linear-gradient(180deg, #fff5f5 0%, #ffe9e9 100%);
}

.publish-time-video-card--changed {
  border-color: #95d475;
  background: linear-gradient(180deg, #f3fff4 0%, #eafbe9 100%);
}

.publish-time-video-card--selected {
  border-color: #409eff;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.16);
}

.publish-time-drop-active {
  border-color: #409eff !important;
  box-shadow: 0 0 0 2px rgba(64, 158, 255, 0.14);
}

.publish-time-drag-ghost {
  position: fixed;
  z-index: 9999;
  pointer-events: none;
  max-width: 240px;
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(64, 158, 255, 0.92);
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  line-height: 1.2;
  box-shadow: 0 10px 24px rgba(64, 158, 255, 0.25);
  transform: translate3d(0, 0, 0);
}

.publish-time-video-card-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.publish-time-video-card-title {
  flex: 1 1 auto;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: #303133;
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.publish-time-video-card-season {
  flex: 0 0 auto;
  max-width: 42%;
  font-size: 11px;
  color: #909399;
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.publish-time-video-card-meta {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  color: #606266;
  font-size: 11px;
  line-height: 1.2;
}

.publish-time-video-card-meta span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.publish-date-range-editor {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 2px 6px;
  border-radius: 6px;
  background: #ffffff;
  border: 1px solid #e8edf5;
}

.publish-date-range-label {
  font-size: 11px;
  color: #909399;
  font-weight: 500;
  white-space: nowrap;
}

.publish-date-range-input {
  width: 112px;
}

.publish-submit-interval-input {
  width: 68px;
}

.publish-date-range-input :deep(.el-input__wrapper),
.publish-submit-interval-input :deep(.el-input__wrapper) {
  box-shadow: none;
  background: transparent;
  padding-left: 4px;
  padding-right: 4px;
}

.publish-time-submit-bar {
  margin-top: auto;
  border-top: 1px solid #f0f2f5;
  padding: 10px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  background: #fff;
}

.publish-time-submit-meta {
  display: flex;
  gap: 10px;
  color: #606266;
  font-size: 12px;
  flex-wrap: wrap;
}

@media (max-width: 1100px) {
  .publish-time-editor {
    grid-template-columns: 1fr;
  }
}
</style>

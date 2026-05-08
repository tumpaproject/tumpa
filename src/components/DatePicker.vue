<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue'

const props = defineProps({
  modelValue: {
    type: String,
    default: ''
  },
  minDate: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['update:modelValue'])

const isOpen = ref(false)
const pickerRef = ref(null)
const triggerRef = ref(null)
const dropdownRef = ref(null)
const currentMonth = ref(new Date())
const focusedDayIndex = ref(-1)
const dropdownStyle = ref({})

const DROPDOWN_WIDTH = 280
const DROPDOWN_HEIGHT = 320
const GAP = 4

function positionDropdown() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  const viewportW = window.innerWidth
  const viewportH = window.innerHeight

  let left = rect.left
  if (left + DROPDOWN_WIDTH > viewportW - 8) {
    left = Math.max(8, viewportW - DROPDOWN_WIDTH - 8)
  }

  const spaceBelow = viewportH - rect.bottom
  const spaceAbove = rect.top
  const openUpward = spaceBelow < DROPDOWN_HEIGHT + GAP && spaceAbove > spaceBelow
  const top = openUpward
    ? Math.max(8, rect.top - DROPDOWN_HEIGHT - GAP)
    : rect.bottom + GAP

  dropdownStyle.value = {
    position: 'fixed',
    top: `${top}px`,
    left: `${left}px`,
    width: `${DROPDOWN_WIDTH}px`
  }
}

const selectedDate = computed(() => {
  if (props.modelValue) {
    return new Date(props.modelValue + 'T00:00:00')
  }
  return null
})

const displayDate = computed(() => {
  if (props.modelValue) {
    return props.modelValue.replace(/-/g, '/')
  }
  return 'YYYY / MM / DD'
})

const minDateObj = computed(() => {
  if (props.minDate) {
    return new Date(props.minDate + 'T00:00:00')
  }
  return null
})

const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
  'July', 'August', 'September', 'October', 'November', 'December']

const dayNames = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa']

const currentMonthName = computed(() => {
  return `${monthNames[currentMonth.value.getMonth()]} ${currentMonth.value.getFullYear()}`
})

const calendarDays = computed(() => {
  const year = currentMonth.value.getFullYear()
  const month = currentMonth.value.getMonth()

  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)

  const days = []

  for (let i = 0; i < firstDay.getDay(); i++) {
    days.push({ day: '', disabled: true, date: null })
  }

  for (let d = 1; d <= lastDay.getDate(); d++) {
    const date = new Date(year, month, d)
    const dateStr = formatDate(date)
    const isDisabled = minDateObj.value && date < minDateObj.value
    const isSelected = props.modelValue === dateStr

    days.push({
      day: d,
      disabled: isDisabled,
      date: dateStr,
      isSelected
    })
  }

  return days
})

function formatDate(date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function togglePicker() {
  isOpen.value = !isOpen.value
  if (isOpen.value) {
    if (selectedDate.value) {
      currentMonth.value = new Date(selectedDate.value)
    } else {
      currentMonth.value = new Date()
    }
    focusedDayIndex.value = -1
    positionDropdown()
    nextTick(() => {
      positionDropdown()
      const selectedIdx = calendarDays.value.findIndex(d => d.isSelected)
      const targetIdx = selectedIdx !== -1 ? selectedIdx : calendarDays.value.findIndex(d => d.day && !d.disabled)
      if (targetIdx !== -1) {
        focusedDayIndex.value = targetIdx
        focusDayAtIndex(targetIdx)
      }
    })
  }
}

function closePicker() {
  isOpen.value = false
  focusedDayIndex.value = -1
  nextTick(() => triggerRef.value?.focus())
}

function selectDate(dateStr) {
  if (dateStr) {
    emit('update:modelValue', dateStr)
    closePicker()
  }
}

function prevMonth() {
  const newDate = new Date(currentMonth.value)
  newDate.setMonth(newDate.getMonth() - 1)
  currentMonth.value = newDate
  focusedDayIndex.value = -1
}

function nextMonth() {
  const newDate = new Date(currentMonth.value)
  newDate.setMonth(newDate.getMonth() + 1)
  currentMonth.value = newDate
  focusedDayIndex.value = -1
}

function getDayTabindex(item, index) {
  if (!item.day || item.disabled) return -1
  if (focusedDayIndex.value === index) return 0
  if (focusedDayIndex.value === -1 && item.isSelected) return 0
  if (focusedDayIndex.value === -1 && !calendarDays.value.some(d => d.isSelected)) {
    const firstAvail = calendarDays.value.findIndex(d => d.day && !d.disabled)
    if (firstAvail === index) return 0
  }
  return -1
}

function focusDayAtIndex(index) {
  nextTick(() => {
    const grid = dropdownRef.value?.querySelector('.calendar-grid')
    if (!grid) return
    const buttons = grid.querySelectorAll('button.day-cell')
    if (buttons[index]) buttons[index].focus()
  })
}

function findNextAvailable(fromIndex, direction) {
  const days = calendarDays.value
  let i = fromIndex + direction
  while (i >= 0 && i < days.length) {
    if (days[i].day && !days[i].disabled) return i
    i += direction
  }
  return -1
}

function handleDayKeydown(event, index) {
  const { key } = event
  let newIndex = -1

  if (key === 'ArrowLeft') {
    newIndex = findNextAvailable(index, -1)
  } else if (key === 'ArrowRight') {
    newIndex = findNextAvailable(index, 1)
  } else if (key === 'ArrowUp') {
    // Move up one week
    const target = index - 7
    if (target >= 0) {
      const item = calendarDays.value[target]
      if (item && item.day && !item.disabled) newIndex = target
    }
  } else if (key === 'ArrowDown') {
    // Move down one week
    const target = index + 7
    if (target < calendarDays.value.length) {
      const item = calendarDays.value[target]
      if (item && item.day && !item.disabled) newIndex = target
    }
  } else if (key === 'Home') {
    // First day of this week row
    const weekStart = index - (index % 7)
    newIndex = findNextAvailable(weekStart - 1, 1)
    if (newIndex > index) newIndex = -1 // Don't go forward
  } else if (key === 'End') {
    // Last day of this week row
    const weekEnd = index - (index % 7) + 6
    const maxEnd = Math.min(weekEnd, calendarDays.value.length - 1)
    newIndex = findNextAvailable(maxEnd + 1, -1)
    if (newIndex < index) newIndex = -1 // Don't go backward
  } else if (key === 'PageUp') {
    event.preventDefault()
    prevMonth()
    nextTick(() => {
      const targetIdx = calendarDays.value.findIndex(d => d.day && !d.disabled)
      if (targetIdx !== -1) {
        focusedDayIndex.value = targetIdx
        focusDayAtIndex(targetIdx)
      }
    })
    return
  } else if (key === 'PageDown') {
    event.preventDefault()
    nextMonth()
    nextTick(() => {
      const targetIdx = calendarDays.value.findIndex(d => d.day && !d.disabled)
      if (targetIdx !== -1) {
        focusedDayIndex.value = targetIdx
        focusDayAtIndex(targetIdx)
      }
    })
    return
  } else if (key === 'Enter' || key === ' ') {
    event.preventDefault()
    const item = calendarDays.value[index]
    if (item && !item.disabled && item.date) selectDate(item.date)
    return
  } else if (key === 'Escape') {
    event.preventDefault()
    closePicker()
    return
  } else {
    return
  }

  if (newIndex !== -1) {
    event.preventDefault()
    focusedDayIndex.value = newIndex
    focusDayAtIndex(newIndex)
  }
}

function handleClickOutside(event) {
  const insideTrigger = pickerRef.value && pickerRef.value.contains(event.target)
  const insideDropdown = dropdownRef.value && dropdownRef.value.contains(event.target)
  if (!insideTrigger && !insideDropdown) {
    isOpen.value = false
    focusedDayIndex.value = -1
  }
}

function handleKeydown(event) {
  if (event.key === 'Escape' && isOpen.value) {
    closePicker()
  }
}

function handleReposition() {
  if (isOpen.value) positionDropdown()
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
  window.addEventListener('resize', handleReposition)
  window.addEventListener('scroll', handleReposition, true)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('resize', handleReposition)
  window.removeEventListener('scroll', handleReposition, true)
})
</script>

<template>
  <div class="date-picker" ref="pickerRef">
    <button
      type="button"
      class="date-input"
      ref="triggerRef"
      @click="togglePicker"
      aria-haspopup="dialog"
      :aria-expanded="isOpen"
      :aria-label="modelValue ? 'Change date, currently ' + displayDate : 'Choose a date'"
    >
      <span :class="{ placeholder: !modelValue }" aria-hidden="true">{{ displayDate }}</span>
      <svg class="calendar-icon" aria-hidden="true" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="16" y1="2" x2="16" y2="6"></line>
        <line x1="8" y1="2" x2="8" y2="6"></line>
        <line x1="3" y1="10" x2="21" y2="10"></line>
      </svg>
    </button>

    <Teleport to="body">
      <div
        v-if="isOpen"
        ref="dropdownRef"
        class="calendar-dropdown"
        role="dialog"
        aria-label="Choose date"
        :style="dropdownStyle"
      >
        <div class="calendar-header">
          <button type="button" class="nav-btn" @click="prevMonth" aria-label="Previous month">&lt;</button>
          <span class="month-label" aria-live="polite">{{ currentMonthName }}</span>
          <button type="button" class="nav-btn" @click="nextMonth" aria-label="Next month">&gt;</button>
        </div>

        <div class="calendar-grid" role="grid" aria-label="Calendar">
          <div v-for="day in dayNames" :key="day" class="day-header" role="columnheader">{{ day }}</div>
          <button
            v-for="(item, index) in calendarDays"
            :key="index"
            type="button"
            class="day-cell"
            :class="{
              disabled: item.disabled,
              selected: item.isSelected,
              empty: !item.day
            }"
            :aria-selected="item.isSelected || undefined"
            :aria-disabled="item.disabled || undefined"
            :disabled="item.disabled || !item.day"
            :tabindex="getDayTabindex(item, index)"
            @click="!item.disabled && selectDate(item.date)"
            @keydown="handleDayKeydown($event, index)"
          >
            {{ item.day }}
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.date-picker {
  position: relative;
  width: 100%;
}

.date-input {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  background: var(--color-bg);
  cursor: pointer;
  font-size: 14px;
  font-family: var(--font-family);
  color: var(--color-text);
  text-align: left;
}

.date-input:hover {
  border-color: var(--color-sidebar);
}

.date-input .placeholder {
  color: var(--color-text-muted);
}

.calendar-icon {
  width: 18px;
  height: 18px;
  color: var(--color-text-muted);
}

.calendar-dropdown {
  background: white;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 12px;
  z-index: 1000;
  font-family: var(--font-family);
}

.calendar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.nav-btn {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  padding: 6px 10px;
  cursor: pointer;
  font-size: 14px;
  font-family: var(--font-family);
}

.nav-btn:hover {
  background: var(--color-bg-light);
}

.month-label {
  font-weight: 500;
  font-size: 14px;
}

.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 2px;
}

.day-header {
  text-align: center;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-muted);
  padding: 8px 0;
}

.day-cell {
  text-align: center;
  padding: 8px;
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
  background: none;
  border: none;
  font-family: var(--font-family);
  color: var(--color-text);
}

.day-cell:hover:not(:disabled):not(.empty) {
  background: #E8E0F0;
}

.day-cell.selected {
  background: var(--color-sidebar);
  color: white;
}

.day-cell:disabled {
  color: #ccc;
  cursor: not-allowed;
}

.day-cell.empty {
  cursor: default;
  visibility: hidden;
}
</style>

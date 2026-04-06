<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'

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
const currentMonth = ref(new Date())

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
  if (isOpen.value && selectedDate.value) {
    currentMonth.value = new Date(selectedDate.value)
  } else if (isOpen.value) {
    currentMonth.value = new Date()
  }
}

function closePicker() {
  isOpen.value = false
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
}

function nextMonth() {
  const newDate = new Date(currentMonth.value)
  newDate.setMonth(newDate.getMonth() + 1)
  currentMonth.value = newDate
}

function handleClickOutside(event) {
  if (pickerRef.value && !pickerRef.value.contains(event.target)) {
    closePicker()
  }
}

function handleKeydown(event) {
  if (event.key === 'Escape') {
    closePicker()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="date-picker" ref="pickerRef">
    <div class="date-input" @click="togglePicker">
      <span :class="{ placeholder: !modelValue }">{{ displayDate }}</span>
      <svg class="calendar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
        <line x1="16" y1="2" x2="16" y2="6"></line>
        <line x1="8" y1="2" x2="8" y2="6"></line>
        <line x1="3" y1="10" x2="21" y2="10"></line>
      </svg>
    </div>

    <div v-if="isOpen" class="calendar-dropdown">
      <div class="calendar-header">
        <button type="button" class="nav-btn" @click="prevMonth">&lt;</button>
        <span class="month-label">{{ currentMonthName }}</span>
        <button type="button" class="nav-btn" @click="nextMonth">&gt;</button>
      </div>

      <div class="calendar-grid">
        <div v-for="day in dayNames" :key="day" class="day-header">{{ day }}</div>
        <div
          v-for="(item, index) in calendarDays"
          :key="index"
          class="day-cell"
          :class="{
            disabled: item.disabled,
            selected: item.isSelected,
            empty: !item.day
          }"
          @click="!item.disabled && selectDate(item.date)"
        >
          {{ item.day }}
        </div>
      </div>
    </div>
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
  padding: 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  background: var(--color-bg);
  cursor: pointer;
  font-size: 14px;
  font-family: var(--font-family);
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
  position: absolute;
  bottom: 0;
  left: 55%;
  background: white;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 12px;
  z-index: 1000;
  width: 280px;
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
  padding: 4px 10px;
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
}

.day-cell:hover:not(.disabled):not(.empty) {
  background: #E8E0F0;
}

.day-cell.selected {
  background: var(--color-sidebar);
  color: white;
}

.day-cell.disabled {
  color: #ccc;
  cursor: not-allowed;
}

.day-cell.empty {
  cursor: default;
}
</style>

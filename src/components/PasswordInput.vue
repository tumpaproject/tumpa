<script setup>
import { ref } from 'vue'
import eyeVisible from '@/assets/icons/eye_visible.svg'
import eyeHidden from '@/assets/icons/eye_hidden.svg'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  modelValue: { type: String, default: '' },
  placeholder: { type: String, default: '' },
  name: { type: String, default: () => `pw-${Math.random().toString(36).slice(2)}` },
})

const emit = defineEmits(['update:modelValue'])
const isVisible = ref(false)

function toggleVisibility() {
  isVisible.value = !isVisible.value
}

function onBlur() {
  isVisible.value = false
}
</script>

<template>
  <div class="password-input">
    <input
      v-bind="$attrs"
      :type="isVisible ? 'text' : 'password'"
      :value="modelValue"
      :placeholder="placeholder"
      :name="props.name"
      @input="emit('update:modelValue', $event.target.value)"
      @blur="onBlur"
      autocomplete="off"
      data-form-type="other"
      data-lpignore="true"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
    />
    <button
      type="button"
      class="eye-btn"
      @click="toggleVisibility"
    >
      <img :src="isVisible ? eyeVisible : eyeHidden" alt="Toggle password visibility" />
    </button>
  </div>
</template>

<style scoped>
.password-input {
  position: relative;
  width: 100%;
}

.password-input input {
  width: 100%;
  padding: 10px 44px 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 6px;
  outline: none;
  background: var(--color-bg);
  font-size: 14px;
  font-family: var(--font-family);
  transition: border-color 0.15s;
}

.password-input input:focus {
  border-color: var(--color-sidebar);
  box-shadow: 0 0 0 1px var(--color-sidebar);
}

.eye-btn {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
}

.eye-btn {
  color: var(--color-sidebar);
}

.eye-btn img {
  width: 20px;
  height: 20px;
  opacity: 0.5;
}

.eye-btn:hover img {
  opacity: 0.8;
}
</style>

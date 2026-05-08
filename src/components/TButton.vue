<script setup>
defineProps({
  variant: {
    type: String,
    default: 'default',
    validator: (v) => ['default', 'green', 'white', 'red', 'red-alt', 'transparent'].includes(v),
  },
  icon: { type: String, default: '' },
  thin: { type: Boolean, default: false },
  disabled: { type: Boolean, default: false },
})

defineEmits(['click'])
</script>

<template>
  <button
    class="t-button"
    :class="[`t-button--${variant}`, { 't-button--thin': thin }]"
    :disabled="disabled"
    @click="$emit('click')"
  >
    <img v-if="icon" :src="icon" alt="" class="t-button-icon" />
    <slot />
  </button>
</template>

<style scoped>
.t-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 42px;
  padding: 0 12px;
  border: none;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  font-family: var(--font-family);
  transition: background 0.15s;
}

.t-button--thin {
  height: 34px;
  font-size: 12px;
}

.t-button:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.t-button-icon {
  width: 18px;
  height: 18px;
}

.t-button--thin .t-button-icon {
  width: 16px;
  height: 16px;
}

/* Variants */
.t-button--default {
  background: var(--color-border);
  color: var(--color-text);
}
.t-button--default:hover:not(:disabled) { background: #DADDE2; }

.t-button--green {
  background: var(--color-green);
  color: var(--color-text-dark);
}
.t-button--green:hover:not(:disabled) { background: var(--color-green-hover); }

.t-button--white {
  background: white;
  color: var(--color-text-dark);
  border: 1px solid var(--color-border-input);
}
.t-button--white:hover:not(:disabled) { background: #FAFAFA; }

.t-button--red {
  background: var(--color-red);
  color: white;
}
.t-button--red:hover:not(:disabled) { background: var(--color-red-hover); }

.t-button--red-alt {
  background: white;
  color: var(--color-red-text);
  border: 1px solid var(--color-border-input);
}
.t-button--red-alt:hover:not(:disabled) { background: #FAFAFA; }

.t-button--transparent {
  background: transparent;
  color: var(--color-indigo);
  height: auto;
  min-height: 24px;
  padding: 0;
  font-size: 12px;
}
.t-button--transparent .t-button-icon {
  width: 16px;
  height: 16px;
}
</style>

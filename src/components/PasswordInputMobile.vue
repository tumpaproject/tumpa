<script setup>
// Mobile-sized password / PIN input with a show/hide eye toggle.
// Same affordance as the desktop PasswordInput, but matched to the
// mobile-input sizing used across UploadToCardMobile / EditName /
// EditUrl / ChangeUserPin / ChangeAdminPin (min-height 44, 16px
// font, 10px rounding). Defensive visibility reset on blur so the
// PIN never stays visible once the user moves away.
import { ref } from 'vue'
import eyeVisible from '@/assets/icons/eye_visible.svg'
import eyeHidden from '@/assets/icons/eye_hidden.svg'

defineOptions({ inheritAttrs: false })

const props = defineProps({
  modelValue: { type: String, default: '' },
  id: { type: String, default: '' },
  autocomplete: { type: String, default: 'current-password' },
  inputmode: { type: String, default: '' },
})

const emit = defineEmits(['update:modelValue'])
const visible = ref(false)

function onBlur() { visible.value = false }
</script>

<template>
  <div class="mobile-pw">
    <input
      v-bind="$attrs"
      :id="id"
      :type="visible ? 'text' : 'password'"
      :value="modelValue"
      :autocomplete="autocomplete"
      :inputmode="inputmode || undefined"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      @input="emit('update:modelValue', $event.target.value)"
      @blur="onBlur"
    />
    <button
      type="button"
      class="eye-btn"
      :aria-label="visible ? 'Hide' : 'Show'"
      :aria-pressed="visible"
      @click="visible = !visible"
    >
      <img :src="visible ? eyeVisible : eyeHidden" alt="" />
    </button>
  </div>
</template>

<style scoped>
.mobile-pw {
  position: relative;
  width: 100%;
}

.mobile-pw input {
  width: 100%;
  min-height: 44px;
  padding: 10px 48px 10px 12px;
  border: 1px solid var(--color-border-input);
  border-radius: 10px;
  font-size: 16px;
  font-family: var(--font-family);
  background: #fff;
  box-sizing: border-box;
}

.mobile-pw input:focus {
  outline: 2px solid var(--color-sidebar-focus);
  outline-offset: 1px;
}

.eye-btn {
  position: absolute;
  right: 4px;
  top: 50%;
  transform: translateY(-50%);
  min-width: 44px;
  min-height: 44px;
  background: transparent;
  border: none;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.eye-btn img {
  width: 22px;
  height: 22px;
  opacity: 0.55;
}

.eye-btn:active img {
  opacity: 0.9;
}
</style>

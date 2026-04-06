<script setup>
import TButton from './TButton.vue'
import keyIconSvg from '@/assets/icons/keyIcon.svg'
import cardPurpleSvg from '@/assets/icons/card_purple.svg'
import exportPurpleSvg from '@/assets/icons/export_purple.svg'
import detailsPurpleSvg from '@/assets/icons/details_purple.svg'
import deletePurpleSvg from '@/assets/icons/delete_purple.svg'

const props = defineProps({
  keyData: { type: Object, required: true },
})

defineEmits(['details', 'upload', 'export', 'delete'])

const isExpired = (() => {
  if (props.keyData.expiration_time === 'Never') return false
  const expDate = new Date(props.keyData.expiration_time)
  return expDate < new Date()
})()
</script>

<template>
  <div class="key-item" :class="{ 'key-item--expired': isExpired, 'key-item--revoked': keyData.is_revoked }">
    <div class="key-row">
      <img :src="keyIconSvg" alt="" class="key-icon" />
      <span class="key-fingerprint">{{ keyData.fingerprint }}</span>
      <span class="key-type-tag">{{ keyData.key_type }}</span>
      <span v-if="keyData.is_revoked" class="revoked-tag">REVOKED</span>
    </div>

    <div class="key-dates">
      <span>Created on: <strong>{{ keyData.creation_time }}</strong></span>
      <span v-if="keyData.is_revoked && keyData.revocation_time" class="expired-text">
        Revoked on: <strong>{{ keyData.revocation_time }}</strong>
      </span>
      <span v-else-if="isExpired" class="expired-text">
        Expired on: <strong>{{ keyData.expiration_time }}</strong>
      </span>
      <span v-else>Expires on: <strong>{{ keyData.expiration_time }}</strong></span>
    </div>

    <div class="key-uids">
      <span
        v-for="(uid, i) in keyData.user_ids"
        :key="i"
        class="uid-pill"
      >
        <strong>{{ uid.name }}</strong> {{ uid.email ? `<${uid.email}>` : '' }}
      </span>
    </div>

    <div class="key-actions">
      <TButton variant="transparent" :icon="detailsPurpleSvg" @click="$emit('details')">Details</TButton>
      <TButton variant="transparent" :icon="cardPurpleSvg" @click="$emit('upload')" :disabled="keyData.is_revoked">Send Key to Card</TButton>
      <TButton variant="transparent" :icon="exportPurpleSvg" @click="$emit('export')">Export Public key</TButton>
      <TButton variant="transparent" :icon="deletePurpleSvg" @click="$emit('delete')">Delete</TButton>
    </div>
  </div>
</template>

<style scoped>
.key-item {
  background: var(--color-bg-light);
  border: 1px solid var(--color-border);
  border-radius: 5px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.key-item--revoked {
  background: var(--color-expired-bg);
  border-color: var(--color-expired-border);
  opacity: 0.7;
}

.key-type-tag {
  background: var(--color-border);
  color: var(--color-text-muted);
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 3px;
}

.revoked-tag {
  background: var(--color-red);
  color: white;
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 3px;
  margin-left: auto;
}

.key-item--expired {
  background: var(--color-expired-bg);
  border-color: var(--color-expired-border);
  background-image: repeating-linear-gradient(
    135deg,
    transparent,
    transparent 10px,
    rgba(252, 165, 165, 0.1) 10px,
    rgba(252, 165, 165, 0.1) 20px
  );
}

.key-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.key-icon {
  width: 18px;
  height: 18px;
}

.key-fingerprint {
  font-size: 14px;
  font-weight: 600;
  line-height: 1.3;
}

.key-dates {
  display: flex;
  gap: 20px;
  font-size: 12px;
  line-height: 1.5;
}

.key-dates strong {
  font-weight: 500;
}

.expired-text {
  color: var(--color-expired-text);
}

.key-uids {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.uid-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--color-border);
  border-radius: 16px;
  padding: 4px 12px;
  font-size: 14px;
}

.uid-pill strong {
  font-weight: 500;
}

.key-actions {
  display: flex;
  gap: 20px;
}
</style>

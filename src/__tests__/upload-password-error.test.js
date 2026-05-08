/// Tests for inline password error display during card upload.
///
/// Regression: Previously, a wrong password during card upload showed a
/// browser alert() dialog and navigated away, losing the typed password.
/// Now the error is shown inline next to "Key Password:" in red, the
/// password is preserved, and the error clears when the user types.

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { setActivePinia, createPinia } from 'pinia'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

import UploadToCardView from '@/views/UploadToCardView.vue'

let router

beforeEach(() => {
  setActivePinia(createPinia())
  router = createRouter({
    history: createWebHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/keys', component: { template: '<div />' } },
      { path: '/card/upload', name: 'upload-to-card', component: UploadToCardView },
    ],
  })
  vi.mocked(invoke).mockClear()
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === 'get_available_subkeys') {
      return {
        primary_can_sign: true,
        signing_subkey: false,
        encryption: true,
        authentication: true,
      }
    }
    if (cmd === 'upload_key_to_card') {
      throw new Error('Incorrect key password.')
    }
    return undefined
  })
  // Suppress confirm dialog in tests
  globalThis.confirm = vi.fn().mockReturnValue(true)
})

async function mountUploadView() {
  await router.push({ name: 'upload-to-card', query: { fingerprint: 'ABCD1234' } })
  await router.isReady()
  const wrapper = mount(UploadToCardView, {
    global: { plugins: [router] },
  })
  await vi.dynamicImportSettled()
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  return wrapper
}

describe('Upload password error handling', () => {
  it('shows error inline instead of alert on wrong password', async () => {
    const wrapper = await mountUploadView()

    // Fill password
    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('wrongpass')

    // Click upload
    const uploadBtn = wrapper.findAll('button').find(b => b.text().includes('Upload'))
    await uploadBtn.trigger('click')

    // Wait for async invoke
    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // Error should be shown inline
    const errorText = wrapper.find('.error-text')
    expect(errorText.exists()).toBe(true)
    expect(errorText.text()).toContain('Incorrect key password')
  })

  it('preserves password in input after error', async () => {
    const wrapper = await mountUploadView()

    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('wrongpass')

    const uploadBtn = wrapper.findAll('button').find(b => b.text().includes('Upload'))
    await uploadBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // Password should still be in the input
    expect(pwInput.element.value).toBe('wrongpass')
  })

  it('clears error when user types new password', async () => {
    const wrapper = await mountUploadView()

    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('wrongpass')

    const uploadBtn = wrapper.findAll('button').find(b => b.text().includes('Upload'))
    await uploadBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // Error should exist
    expect(wrapper.find('.error-text').exists()).toBe(true)

    // Type new password — triggers PasswordInput's @input → emit('update:modelValue')
    // which updates the parent's password ref → watch clears errorMessage
    const pwComponent = wrapper.findComponent({ name: 'PasswordInput' })
    pwComponent.vm.$emit('update:modelValue', 'newpass')
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.error-text').exists()).toBe(false)
  })

  it('stays on same page after error (no navigation)', async () => {
    const wrapper = await mountUploadView()

    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('wrongpass')

    const uploadBtn = wrapper.findAll('button').find(b => b.text().includes('Upload'))
    await uploadBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // Should still be on the upload page
    expect(router.currentRoute.value.path).toBe('/card/upload')
  })

  it('shows empty error when no password entered', async () => {
    const wrapper = await mountUploadView()

    // Don't fill password, just click upload
    const uploadBtn = wrapper.findAll('button').find(b => b.text().includes('Upload'))
    await uploadBtn.trigger('click')

    await wrapper.vm.$nextTick()

    const errorText = wrapper.find('.error-text')
    expect(errorText.exists()).toBe(true)
    expect(errorText.text()).toContain('Please enter the key password')
  })
})

/// Tests for smartcard edit views navigating to card details on success.
///
/// Regression: Previously, edit views (name, URL, PINs) showed an alert()
/// on success instead of navigating to the card details screen. This made
/// it hard to verify the change took effect.

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { setActivePinia, createPinia } from 'pinia'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

import { invoke } from '@tauri-apps/api/core'

import EditNameView from '@/views/EditNameView.vue'
import EditUrlView from '@/views/EditUrlView.vue'
import ChangeUserPinView from '@/views/ChangeUserPinView.vue'
import ChangeAdminPinView from '@/views/ChangeAdminPinView.vue'

let router

function createTestRouter() {
  return createRouter({
    history: createWebHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/card', name: 'card-details', component: { template: '<div>card details</div>' } },
      { path: '/card/edit-name', name: 'edit-name', component: EditNameView },
      { path: '/card/edit-url', name: 'edit-url', component: EditUrlView },
      { path: '/card/change-user-pin', name: 'change-user-pin', component: ChangeUserPinView },
      { path: '/card/change-admin-pin', name: 'change-admin-pin', component: ChangeAdminPinView },
    ],
  })
}

beforeEach(() => {
  setActivePinia(createPinia())
  router = createTestRouter()
  vi.mocked(invoke).mockClear()
  // Mock get_card_details for fetchCardDetails
  vi.mocked(invoke).mockImplementation(async (cmd) => {
    if (cmd === 'get_card_details') {
      return {
        ident: '0006:00000001',
        serial_number: '00000001',
        cardholder_name: '',
        public_key_url: '',
        pin_retry_counter: 3,
        reset_code_retry_counter: 0,
        admin_pin_retry_counter: 3,
        signature_fingerprint: null,
        encryption_fingerprint: null,
        authentication_fingerprint: null,
        manufacturer: '0006',
        manufacturer_name: 'Yubico AB',
      }
    }
    if (cmd === 'is_card_connected') return true
    return undefined
  })
})

describe('EditNameView', () => {
  it('navigates to /card on successful name update', async () => {
    await router.push('/card/edit-name')
    await router.isReady()

    const wrapper = mount(EditNameView, {
      global: { plugins: [router] },
    })

    // Fill in form
    const inputs = wrapper.findAll('input')
    await inputs[0].setValue('Test Name')
    // PasswordInput wraps an input
    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('12345678')

    // Click save
    const saveBtn = wrapper.findAll('button').find(b => b.text().includes('Save'))
    await saveBtn.trigger('click')

    // Wait for async
    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('update_card_name', {
      name: 'Test Name',
      adminPin: '12345678',
    })
    expect(router.currentRoute.value.path).toBe('/card')
  })
})

describe('EditUrlView', () => {
  it('navigates to /card on successful URL update', async () => {
    await router.push('/card/edit-url')
    await router.isReady()

    const wrapper = mount(EditUrlView, {
      global: { plugins: [router] },
    })

    const inputs = wrapper.findAll('input')
    await inputs[0].setValue('https://keys.openpgp.org')
    const pwInput = wrapper.find('.password-input input')
    await pwInput.setValue('12345678')

    const saveBtn = wrapper.findAll('button').find(b => b.text().includes('Save'))
    await saveBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('update_card_url', {
      url: 'https://keys.openpgp.org',
      adminPin: '12345678',
    })
    expect(router.currentRoute.value.path).toBe('/card')
  })
})

describe('ChangeUserPinView', () => {
  it('navigates to /card on successful PIN change', async () => {
    await router.push('/card/change-user-pin')
    await router.isReady()

    const wrapper = mount(ChangeUserPinView, {
      global: { plugins: [router] },
    })

    const pwInputs = wrapper.findAll('.password-input input')
    await pwInputs[0].setValue('12345678')
    await pwInputs[1].setValue('654321')

    const saveBtn = wrapper.findAll('button').find(b => b.text().includes('Save'))
    await saveBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('change_user_pin', {
      adminPin: '12345678',
      newPin: '654321',
    })
    expect(router.currentRoute.value.path).toBe('/card')
  })
})

describe('ChangeAdminPinView', () => {
  it('navigates to /card on successful admin PIN change', async () => {
    await router.push('/card/change-admin-pin')
    await router.isReady()

    const wrapper = mount(ChangeAdminPinView, {
      global: { plugins: [router] },
    })

    const pwInputs = wrapper.findAll('.password-input input')
    await pwInputs[0].setValue('12345678')
    await pwInputs[1].setValue('87654321')

    const saveBtn = wrapper.findAll('button').find(b => b.text().includes('Save'))
    await saveBtn.trigger('click')

    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()

    expect(invoke).toHaveBeenCalledWith('change_admin_pin', {
      currentPin: '12345678',
      newPin: '87654321',
    })
    expect(router.currentRoute.value.path).toBe('/card')
  })
})

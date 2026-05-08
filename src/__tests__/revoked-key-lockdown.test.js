/// Tests that revoked keys have all modification actions disabled.
///
/// Regression: Previously, revoked keys could still be uploaded to card,
/// have their password changed, UIDs added, and expiry modified. All these
/// actions should be disabled when a key is revoked.

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { setActivePinia, createPinia } from 'pinia'

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

// Mock Tauri dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
  open: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'

import KeyDetailsView from '@/views/KeyDetailsView.vue'
import KeyItem from '@/components/KeyItem.vue'

function makeRevokedKey() {
  return {
    fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234',
    key_id: 'ABCD1234',
    creation_time: '01 Apr 2026',
    expiration_time: 'Never',
    user_ids: [
      { name: 'Test User', email: 'test@example.com', revoked: false, revocation_time: null },
    ],
    is_secret: true,
    is_revoked: true,
    revocation_time: '06 Apr 2026 13:46',
    subkeys: [
      { fingerprint: 'SUB1', key_type: 'encryption', creation_time: '01 Apr 2026', expiration_time: 'Never', is_revoked: false },
      { fingerprint: 'SUB2', key_type: 'authentication', creation_time: '01 Apr 2026', expiration_time: 'Never', is_revoked: false },
    ],
  }
}

function makeValidKey() {
  return {
    ...makeRevokedKey(),
    is_revoked: false,
    revocation_time: null,
  }
}

let router

beforeEach(() => {
  setActivePinia(createPinia())
  router = createRouter({
    history: createWebHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/keys', component: { template: '<div />' } },
      { path: '/keys/:fingerprint', name: 'key-details', component: KeyDetailsView, props: true },
      { path: '/keys/:fingerprint/add-uid', name: 'add-uid', component: { template: '<div />' } },
      { path: '/keys/:fingerprint/change-password', name: 'change-key-password', component: { template: '<div />' } },
      { path: '/card/upload', name: 'upload-to-card', component: { template: '<div />' } },
    ],
  })
  vi.mocked(invoke).mockClear()
})

describe('KeyDetailsView with revoked key', () => {
  beforeEach(async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_key_details') return makeRevokedKey()
      if (cmd === 'is_card_connected') return true
      if (cmd === 'list_keys') return [makeRevokedKey()]
      return undefined
    })
  })

  async function mountKeyDetails() {
    await router.push('/keys/ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234')
    await router.isReady()
    const wrapper = mount(KeyDetailsView, {
      global: { plugins: [router] },
      props: { fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234' },
    })
    await wrapper.vm.$nextTick()
    return wrapper
  }

  it('shows revoked banner', async () => {
    const wrapper = await mountKeyDetails()
    const banner = wrapper.find('.revoked-banner')
    expect(banner.exists()).toBe(true)
    expect(banner.text()).toContain('revoked')
    expect(banner.text()).toContain('06 Apr 2026 13:46')
  })

  it('disables Send Key to Card button', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Send Key to Card'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('disables Change Password button in Advanced section', async () => {
    const wrapper = await mountKeyDetails()
    // Open Advanced dropdown
    const advBtn = wrapper.findAll('button').find(b => b.text().includes('Advanced'))
    await advBtn.trigger('click')
    await wrapper.vm.$nextTick()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Change Password'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('disables Revoke Key button', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Revoke Key'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('disables Add new user button', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Add new user'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('keeps Export Public Key enabled', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Export Public Key'))
    expect(btn.attributes('disabled')).toBeUndefined()
  })

  it('keeps Remove enabled', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Remove'))
    expect(btn.attributes('disabled')).toBeUndefined()
  })

  it('disables primary key expiry Change button', async () => {
    const wrapper = await mountKeyDetails()
    const changeBtn = wrapper.findAll('.expiry-change-btn').find(b => b.text() === 'Change')
    if (changeBtn) {
      expect(changeBtn.attributes('disabled')).toBeDefined()
    }
  })

  it('disables subkey Change expiry button', async () => {
    const wrapper = await mountKeyDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Change expiry'))
    expect(btn.attributes('disabled')).toBeDefined()
  })
})

describe('KeyDetailsView with valid key', () => {
  beforeEach(async () => {
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_key_details') return makeValidKey()
      if (cmd === 'is_card_connected') return true
      if (cmd === 'list_keys') return [makeValidKey()]
      return undefined
    })
  })

  it('does not show revoked banner', async () => {
    await router.push('/keys/ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234')
    await router.isReady()
    const wrapper = mount(KeyDetailsView, {
      global: { plugins: [router] },
      props: { fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234' },
    })
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.revoked-banner').exists()).toBe(false)
  })

  it('enables modification buttons for valid key', async () => {
    await router.push('/keys/ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234')
    await router.isReady()
    const wrapper = mount(KeyDetailsView, {
      global: { plugins: [router] },
      props: { fingerprint: 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234' },
    })
    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()

    // Open Advanced dropdown
    const advBtn = wrapper.findAll('button').find(b => b.text().includes('Advanced'))
    await advBtn.trigger('click')
    await wrapper.vm.$nextTick()

    // Change Password in Advanced should be enabled
    const changePwBtn = wrapper.findAll('button').find(b => b.text().includes('Change Password'))
    expect(changePwBtn).toBeDefined()
    expect(changePwBtn.attributes('disabled')).toBeUndefined()

    // Add new user should be enabled
    const addUserBtn = wrapper.findAll('button').find(b => b.text().includes('Add new user'))
    expect(addUserBtn).toBeDefined()
    expect(addUserBtn.attributes('disabled')).toBeUndefined()
  })
})

describe('KeyItem with revoked key', () => {
  it('disables Send Key to Card in key list', () => {
    const wrapper = mount(KeyItem, {
      props: { keyData: makeRevokedKey() },
    })
    const btn = wrapper.findAll('button').find(b => b.text().includes('Send Key to Card'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('shows REVOKED badge', () => {
    const wrapper = mount(KeyItem, {
      props: { keyData: makeRevokedKey() },
    })
    expect(wrapper.find('.revoked-tag').exists()).toBe(true)
    expect(wrapper.find('.revoked-tag').text()).toBe('REVOKED')
  })

  it('does not show REVOKED badge for valid key', () => {
    const wrapper = mount(KeyItem, {
      props: { keyData: makeValidKey() },
    })
    expect(wrapper.find('.revoked-tag').exists()).toBe(false)
  })
})

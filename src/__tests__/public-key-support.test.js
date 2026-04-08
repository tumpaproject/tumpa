/// Tests for public key display and action gating.
///
/// Public keys should show blue styling, PUBLIC badge, and have
/// modification actions (Send to Card, Revoke, Add UID, Change Password)
/// hidden or disabled.

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { setActivePinia, createPinia } from 'pinia'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn(),
  open: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import KeyItem from '@/components/KeyItem.vue'
import KeyDetailsView from '@/views/KeyDetailsView.vue'

function makePublicKey() {
  return {
    fingerprint: 'A85FF376759C994A8A1168D8D8219C8C43F6C5E1',
    key_id: '43F6C5E1',
    creation_time: '01 Apr 2026',
    expiration_time: 'Never',
    key_type: 'Cv25519',
    user_ids: [{ name: 'Contact', email: 'contact@example.com', revoked: false, revocation_time: null }],
    is_secret: false,
    is_revoked: false,
    revocation_time: null,
    subkeys: [
      { fingerprint: 'SUB1', key_type: 'encryption', creation_time: '01 Apr 2026', expiration_time: 'Never', is_revoked: false },
    ],
  }
}

function makeSecretKey() {
  return { ...makePublicKey(), is_secret: true }
}

describe('KeyItem with public key', () => {
  it('shows PUBLIC badge', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makePublicKey() } })
    expect(wrapper.find('.public-tag').exists()).toBe(true)
    expect(wrapper.find('.public-tag').text()).toBe('PUBLIC')
  })

  it('has blue styling', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makePublicKey() } })
    expect(wrapper.find('.key-item--public').exists()).toBe(true)
  })

  it('hides Send Key to Card button', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makePublicKey() } })
    const btn = wrapper.findAll('button').find(b => b.text().includes('Send Key to Card'))
    expect(btn).toBeUndefined()
  })

  it('shows Export and Delete buttons', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makePublicKey() } })
    expect(wrapper.findAll('button').find(b => b.text().includes('Export Public key'))).toBeDefined()
    expect(wrapper.findAll('button').find(b => b.text().includes('Delete'))).toBeDefined()
  })

  it('does not show PRIVATE badge', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makePublicKey() } })
    expect(wrapper.find('.private-tag').exists()).toBe(false)
  })
})

describe('KeyItem with secret key', () => {
  it('shows PRIVATE badge', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makeSecretKey() } })
    expect(wrapper.find('.private-tag').exists()).toBe(true)
    expect(wrapper.find('.private-tag').text()).toBe('PRIVATE')
  })

  it('does not have blue styling', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makeSecretKey() } })
    expect(wrapper.find('.key-item--public').exists()).toBe(false)
  })

  it('shows Send Key to Card button', () => {
    const wrapper = mount(KeyItem, { props: { keyData: makeSecretKey() } })
    const btn = wrapper.findAll('button').find(b => b.text().includes('Send Key to Card'))
    expect(btn).toBeDefined()
  })
})

describe('KeyDetailsView with public key', () => {
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
    vi.mocked(invoke).mockImplementation(async (cmd) => {
      if (cmd === 'get_key_details') return makePublicKey()
      if (cmd === 'list_keys') return [makePublicKey()]
      return undefined
    })
  })

  async function mountDetails() {
    const fp = 'A85FF376759C994A8A1168D8D8219C8C43F6C5E1'
    await router.push(`/keys/${fp}`)
    await router.isReady()
    const wrapper = mount(KeyDetailsView, {
      global: { plugins: [router] },
      props: { fingerprint: fp },
    })
    await vi.dynamicImportSettled()
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    return wrapper
  }

  it('shows public key banner', async () => {
    const wrapper = await mountDetails()
    expect(wrapper.find('.public-banner').exists()).toBe(true)
    expect(wrapper.find('.public-banner').text()).toContain('public key')
  })

  it('hides Send Key to Card button', async () => {
    const wrapper = await mountDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Send Key to Card'))
    expect(btn).toBeUndefined()
  })

  it('hides Revoke Key button', async () => {
    const wrapper = await mountDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Revoke Key'))
    expect(btn).toBeUndefined()
  })

  it('disables Add new user button', async () => {
    const wrapper = await mountDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Add new user'))
    expect(btn.attributes('disabled')).toBeDefined()
  })

  it('keeps Export Public Key enabled', async () => {
    const wrapper = await mountDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Export Public Key'))
    expect(btn).toBeDefined()
    expect(btn.attributes('disabled')).toBeUndefined()
  })

  it('keeps Remove enabled', async () => {
    const wrapper = await mountDetails()
    const btn = wrapper.findAll('button').find(b => b.text().includes('Remove'))
    expect(btn).toBeDefined()
    expect(btn.attributes('disabled')).toBeUndefined()
  })
})

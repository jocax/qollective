/**
 * Vitest Setup File
 *
 * Provides global auto-imports for Vue and Nuxt APIs to match runtime behavior
 */

import { vi } from 'vitest'
import * as vue from 'vue'

// Make Vue composition API functions globally available (like Nuxt does)
Object.assign(global, {
  // Vue reactivity
  ref: vue.ref,
  reactive: vue.reactive,
  computed: vue.computed,
  readonly: vue.readonly,
  watch: vue.watch,
  watchEffect: vue.watchEffect,
  isRef: vue.isRef,
  unref: vue.unref,
  toRef: vue.toRef,
  toRefs: vue.toRefs,
  isReactive: vue.isReactive,
  isReadonly: vue.isReadonly,
  isProxy: vue.isProxy,
  shallowRef: vue.shallowRef,
  triggerRef: vue.triggerRef,
  shallowReactive: vue.shallowReactive,
  shallowReadonly: vue.shallowReadonly,
  toRaw: vue.toRaw,
  markRaw: vue.markRaw,
  effectScope: vue.effectScope,
  getCurrentScope: vue.getCurrentScope,
  onScopeDispose: vue.onScopeDispose,

  // Vue lifecycle
  onBeforeMount: vue.onBeforeMount,
  onMounted: vue.onMounted,
  onBeforeUpdate: vue.onBeforeUpdate,
  onUpdated: vue.onUpdated,
  onBeforeUnmount: vue.onBeforeUnmount,
  onUnmounted: vue.onUnmounted,
  onActivated: vue.onActivated,
  onDeactivated: vue.onDeactivated,
  onErrorCaptured: vue.onErrorCaptured,

  // Vue component
  defineComponent: vue.defineComponent,
  defineAsyncComponent: vue.defineAsyncComponent,
  defineProps: vue.defineProps,
  defineEmits: vue.defineEmits,
  defineExpose: vue.defineExpose,
  useAttrs: vue.useAttrs,
  useSlots: vue.useSlots,

  // Vue injection
  inject: vue.inject,
  provide: vue.provide,

  // Vue current instance
  getCurrentInstance: vue.getCurrentInstance,

  // Vue Suspense
  onServerPrefetch: vue.onServerPrefetch,

  // Nuxt composables stubs
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    go: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    resolve: vi.fn()
  }),
  useRoute: () => ({
    path: '/',
    params: {},
    query: {},
    hash: '',
    fullPath: '/',
    matched: [],
    meta: {},
    name: undefined
  }),
  navigateTo: vi.fn(),
  useHead: vi.fn(),
  useSeoMeta: vi.fn(),
  definePageMeta: vi.fn()
})

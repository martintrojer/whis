import { ref, computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'

export interface AsyncOperationState<T> {
  data: Ref<T | null>
  error: Ref<string | null>
  isLoading: Ref<boolean>
  isSuccess: ComputedRef<boolean>
  isError: ComputedRef<boolean>
}

export interface AsyncOperationReturn<T, Args extends unknown[] = unknown[]> extends AsyncOperationState<T> {
  execute: (...args: Args) => Promise<T | null>
  reset: () => void
}

/**
 * Composable for managing async operation state.
 * Provides consistent loading, error, and success handling.
 *
 * @param operation - Async function to execute
 * @param options - Configuration options
 * @returns State and control functions
 *
 * @example
 * ```ts
 * const { data, isLoading, error, execute } = useAsyncOperation(
 *   async (id: string) => await invoke<User>('get_user', { id })
 * )
 *
 * // In template: v-if="isLoading" / v-if="error" / {{ data }}
 * await execute('user-123')
 * ```
 */
export function useAsyncOperation<T, Args extends unknown[] = unknown[]>(
  operation: (...args: Args) => Promise<T>,
  options: {
    /** Clear error after this many ms (0 = never) */
    errorTimeout?: number
    /** Clear success message after this many ms (0 = never) */
    successTimeout?: number
    /** Called on successful completion */
    onSuccess?: (data: T) => void
    /** Called on error */
    onError?: (error: string) => void
  } = {}
): AsyncOperationReturn<T, Args> {
  const data = ref<T | null>(null) as Ref<T | null>
  const error = ref<string | null>(null)
  const isLoading = ref(false)

  const isSuccess = computed(() => data.value !== null && !error.value && !isLoading.value)
  const isError = computed(() => error.value !== null)

  let errorTimeoutId: ReturnType<typeof setTimeout> | null = null
  let successTimeoutId: ReturnType<typeof setTimeout> | null = null

  function clearTimeouts() {
    if (errorTimeoutId) {
      clearTimeout(errorTimeoutId)
      errorTimeoutId = null
    }
    if (successTimeoutId) {
      clearTimeout(successTimeoutId)
      successTimeoutId = null
    }
  }

  async function execute(...args: Args): Promise<T | null> {
    clearTimeouts()
    isLoading.value = true
    error.value = null

    try {
      const result = await operation(...args)
      data.value = result
      options.onSuccess?.(result)

      if (options.successTimeout && options.successTimeout > 0) {
        successTimeoutId = setTimeout(() => {
          // Optionally reset data after success timeout
        }, options.successTimeout)
      }

      return result
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      error.value = errorMessage
      options.onError?.(errorMessage)

      if (options.errorTimeout && options.errorTimeout > 0) {
        errorTimeoutId = setTimeout(() => {
          error.value = null
        }, options.errorTimeout)
      }

      return null
    } finally {
      isLoading.value = false
    }
  }

  function reset() {
    clearTimeouts()
    data.value = null
    error.value = null
    isLoading.value = false
  }

  return {
    data,
    error,
    isLoading,
    isSuccess,
    isError,
    execute,
    reset,
  }
}

/**
 * Simpler version for operations that just need loading + status message.
 * Common pattern in this codebase.
 *
 * @example
 * ```ts
 * const { status, isLoading, execute } = useStatusOperation(
 *   async () => {
 *     await invoke('save_settings', { settings })
 *     return 'Saved'
 *   },
 *   { successTimeout: 2000 }
 * )
 * ```
 */
export function useStatusOperation(
  operation: () => Promise<string>,
  options: {
    successTimeout?: number
    errorTimeout?: number
  } = {}
) {
  const status = ref('')
  const isLoading = ref(false)

  let timeoutId: ReturnType<typeof setTimeout> | null = null

  async function execute(): Promise<boolean> {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
    }

    isLoading.value = true
    status.value = ''

    try {
      status.value = await operation()

      if (options.successTimeout && options.successTimeout > 0) {
        timeoutId = setTimeout(() => {
          status.value = ''
        }, options.successTimeout)
      }

      return true
    } catch (e) {
      status.value = e instanceof Error ? e.message : String(e)

      if (options.errorTimeout && options.errorTimeout > 0) {
        timeoutId = setTimeout(() => {
          status.value = ''
        }, options.errorTimeout)
      }

      return false
    } finally {
      isLoading.value = false
    }
  }

  return {
    status,
    isLoading,
    execute,
  }
}

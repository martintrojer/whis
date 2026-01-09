/**
 * Suppress intlify HTML warnings during SSR prerendering.
 *
 * Temporary workaround for vue-i18n's `warnHtmlMessage: false` not being
 * respected during SSR. We intentionally use HTML in i18n messages for
 * formatting (<code>, <strong>, <br> tags).
 *
 * TODO: Remove once @nuxtjs/i18n or vue-i18n fixes this.
 * Related: https://github.com/nuxt-modules/i18n/discussions/1968
 */
export default defineNitroPlugin(() => {
  const originalWarn = console.warn
  console.warn = (...args: unknown[]) => {
    const message = args[0]
    if (
      typeof message === 'string'
      && message.includes('[intlify]')
      && message.includes('Detected HTML')
    ) {
      return
    }
    originalWarn.apply(console, args)
  }
})

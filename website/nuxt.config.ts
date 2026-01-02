// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  srcDir: 'app',
  compatibilityDate: '2024-11-01',

  dir: {
    public: '../public',
  },

  // future: {
  //   compatibilityVersion: 4,
  // },

  ssr: true,

  modules: ['@nuxtjs/seo'],

  site: {
    url: 'https://whis.ink',
    name: 'whis',
    description: 'Voice-to-text for Linux',
    defaultLocale: 'en',
  },

  sitemap: {
    strictNuxtContentPaths: true,
  },

  robots: {
    allow: '/',
    sitemap: 'https://whis.ink/sitemap.xml',
  },

  css: [
    '~/assets/tokens.css',
    '~/assets/base.css',
  ],

  app: {
    head: {
      charset: 'utf-8',
      viewport: 'width=device-width, initial-scale=1',
      htmlAttrs: {
        lang: 'en',
      },
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
      ],
    },
    baseURL: '/',
  },

  runtimeConfig: {
    public: {
      appVersion: '0.6.4',
    },
  },

  nitro: {
    static: true,
    prerender: {
      crawlLinks: true,
      routes: ['/', '/downloads', '/cli', '/desktop', '/mobile', '/faq'],
      failOnError: false,
    },
  },

  experimental: {
    payloadExtraction: false,
  },

  typescript: {
    strict: true,
    typeCheck: false, // Disable vite-plugin-checker (removed vue-tsc)
  },

  devtools: { enabled: false },

})

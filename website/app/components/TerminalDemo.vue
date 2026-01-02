<script setup lang="ts">
import { onMounted, ref } from 'vue'

// Terminal state
const lines = ref<string[]>([''])
const showCursor = ref(false)

// Current line index (where we're typing)
const currentLineIndex = ref(0)

const sleep = (ms: number) => new Promise(r => setTimeout(r, ms))

// Helper to get/set current line
function getCurrentLine(): string {
  return lines.value[currentLineIndex.value] || ''
}

function setCurrentLine(text: string) {
  lines.value[currentLineIndex.value] = text
}

// Action handlers
async function typeText(text: string, speed: number) {
  for (const char of text) {
    setCurrentLine(getCurrentLine() + char)
    await sleep(speed)
  }
}

async function backspace(count: number, speed: number) {
  for (let i = 0; i < count; i++) {
    const line = getCurrentLine()
    if (line.length > 0) {
      setCurrentLine(line.slice(0, -1))
      await sleep(speed)
    }
  }
}

function appendText(text: string) {
  setCurrentLine(getCurrentLine() + text)
}

function newLine() {
  lines.value.push('')
  currentLineIndex.value = lines.value.length - 1
}

function clearTerminal() {
  lines.value = ['']
  currentLineIndex.value = 0
}

// Demo definitions
async function runDemo1() {
  // Quick Start: $ whis
  appendText('$ ')
  await typeText('whis', 80)
  await sleep(400)

  newLine()
  appendText('Press Enter to stop recording')

  await sleep(300)
  newLine()
  appendText('Recording...')

  await sleep(800)
  await typeText(' Transcribing...', 25)
  await typeText(' Done', 20)

  await sleep(200)
  newLine()
  appendText('Copied to clipboard')

  await sleep(300)
  newLine()
  appendText('$ ')
  showCursor.value = true

  await sleep(2000)
  showCursor.value = false
}

async function runDemo2() {
  // AI Integration: command substitution with various tools
  appendText('$ ')

  // Claude
  await typeText('claude "$(whis)"', 50)
  await sleep(600)

  // Backspace all, type codex
  await backspace(16, 30) // 'claude "$(whis)"' = 16 chars
  await typeText('codex "$(whis)"', 60)
  await sleep(500)

  // Backspace all, type opencode
  await backspace(15, 30) // 'codex "$(whis)"' = 15 chars
  await typeText('opencode -p "$(whis)"', 60)
  await sleep(500)

  // Backspace all, type gemini
  await backspace(21, 30) // 'opencode -p "$(whis)"' = 21 chars
  await typeText('gemini "$(whis)"', 60)
  await sleep(500)

  // Ctrl+C
  appendText('^C')
  await sleep(400)
}

async function runDemo3() {
  // Provider Config -> Local Setup
  const openaiCmd = 'whis config --openai-key sk-proj-Abc1XyZ9...'
  const mistralCmd = 'whis config --mistral-key Xyz7kQ3rAbC9...'
  const localCmd = 'whis setup local'

  appendText('$ ')
  await typeText(openaiCmd, 40)
  await sleep(600)

  // Delete the whole command after $
  await backspace(openaiCmd.length, 30)

  await typeText(mistralCmd, 40)
  await sleep(600)

  // Delete and type local setup
  await backspace(mistralCmd.length, 30)

  await typeText(localCmd, 60)
  await sleep(400)

  // Show output
  newLine()
  appendText('Local Setup')
  await sleep(300)
  newLine()
  appendText('===========')
  await sleep(500)

  newLine()
  appendText('Step 1: Whisper Model')
  await sleep(400)
  newLine()
  await typeText('Downloading ggml-small.bin...', 25)
  await sleep(800)
  await typeText(' Done', 30)
  await sleep(600)

  newLine()
  appendText('Step 2: Ollama')
  await sleep(400)
  newLine()
  await typeText('Pulling qwen2.5:1.5b...', 30)
  await sleep(800)
  await typeText(' Ready', 30)
  await sleep(600)

  newLine()
  appendText('Setup complete!')
  await sleep(1000)

  newLine()
  appendText('$ ')
  await typeText('clear', 60)
  await sleep(300)
}

async function runDemo4() {
  // Meta Demo: Claude helps you get started
  appendText('$ ')
  await typeText('claude "Get me started with whis --help"', 40)
  showCursor.value = true
  await sleep(10000)
  showCursor.value = false
}

// Main loop
async function runAllDemos() {
  while (true) {
    // Demo 1
    clearTerminal()
    await runDemo1()

    // Demo 2
    clearTerminal()
    await runDemo2()

    // Demo 3
    clearTerminal()
    await runDemo3()

    // Demo 4
    clearTerminal()
    await runDemo4()

    await sleep(500)
  }
}

onMounted(async () => {
  await sleep(800)
  runAllDemos()
})
</script>

<template>
  <div class="terminal">
    <pre><code><template v-for="(line, i) in lines" :key="i"><span v-if="line || i === lines.length - 1" class="line">{{ line }}<span v-if="showCursor && i === lines.length - 1" class="cursor">█</span></span>
</template></code></pre>
  </div>
</template>

<style scoped>
.terminal {
  background: var(--bg-weak);
  border: 1px solid var(--border-weak);
  border-radius: 6px;
  padding: 1.25rem;
  overflow-x: auto;
  min-height: 180px;
}

.terminal pre {
  margin: 0;
}

.terminal code {
  font-family: var(--font);
  font-size: 0.9rem;
  line-height: 1.4;
}

.cursor {
  animation: blink 1s step-end infinite;
}

@keyframes blink {
  50% {
    opacity: 0;
  }
}
</style>

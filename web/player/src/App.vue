<script setup lang="ts">
import GamePage from './pages/GamePage.vue'
import LoginPage from './pages/LoginPage.vue'
import { useSessionStore } from './stores/session'

const session = useSessionStore()
session.restore()
</script>

<template>
  <LoginPage v-if="!session.token" />
  <GamePage v-else />
</template>

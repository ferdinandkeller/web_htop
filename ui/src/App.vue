<template>
  <!-- create an app div that contains everything there is to display -->
  <div class="app">
    <!-- display all the cpus -->
    <div class="cpus">
      <!-- dislpay each individal cpu -->
      <div class="cpu" v-for="(cpu, index) in cpus">
        <!-- display usage as text -->
        <div class="usage-text">{{ cpus_display[index] }}</div>
        <!-- display usage as a loading bar -->
        <div class="usage" :style="{ width: cpu + '%' }"></div>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.app {
  min-width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 2rem;
  user-select: none;

  .cpus {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;

    .cpu {
      width: min(100%, 40rem);
      height: 40px;
      border-radius: 10px;
      display: flex;
      justify-content: center;
      align-items: center;
      overflow: hidden;
      position: relative;
      border: solid 2px var(--border);

      &:not(:last-child) {
        margin-bottom: 1rem;
      }

      .usage-text {
        z-index: 1;
        color: var(--border);
      }

      .usage {
        height: 100%;
        position: absolute;
        top: 0;
        left: 0;
        background-color: var(--loader);
        transition: width .5s ease-out;
      }
    }
  }
}
</style>

<script lang="ts" setup>
import { ref, computed } from 'vue'

let cpus = ref([])
let cpus_display = computed(() => {
  return cpus.value.map((cpu) => {
    let cpu_value = Math.round(cpu * 100) / 100
    let cpu_value_str = cpu_value.toFixed(2)
    cpu_value_str = '0'.repeat(5 - cpu_value_str.length) + cpu_value_str
    return cpu_value_str + '%'
  })
})

function socket_connect() {
  console.log('socket connection try')
  try {
    let socket = new WebSocket('ws://localhost:3000/ws')

    socket.onopen = () => {
    }

    socket.onmessage = (message) => {
      cpus.value = JSON.parse(message.data)
    }

    socket.onerror = () => {
    }

    socket.onclose = () => {
      setTimeout(() => {
        socket_connect()
      }, 1000)
    }
  } catch (e) { }
}

socket_connect()
</script>

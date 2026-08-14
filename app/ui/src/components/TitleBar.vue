<script setup lang="ts">

</script>

<template>
<div class="title-bar" data-tauri-drag-region>
  <button class="close button">X</button>
  <button class="minimize button">_</button>

  <button class="settings button">S</button>
</div>
</template>

<style scoped>
.title-bar {
  background: aqua;
  width: 100%;

  display: flex;
}

.title-bar button {
  width: 26px;
  height: 26px;
  border-radius: 30%;
  color: white;
}
</style>
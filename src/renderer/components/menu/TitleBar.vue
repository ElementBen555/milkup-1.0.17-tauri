<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import AppIcon from "@/renderer/components/ui/AppIcon.vue";
import TabBar from "@/renderer/components/workspace/TabBar.vue";
import MenuDropDown from "./MenuDropDown.vue";

const isWinOrLinux = window.electronAPI.platform === "win32" || window.electronAPI.platform === "linux";

// ─── 窗口拖移（Tauri）───
const barRef = ref<HTMLElement | null>(null);
let dragTimer: ReturnType<typeof setTimeout> | null = null;
function onMouseDown(e: MouseEvent) {
  const t = e.target as HTMLElement;
  if (t.closest(".tabItem,.closeIcon,.window-control-btn,.dropdown-header,.dropdown-content,.menu-option,button,a,input")) return;
  // 200ms 内若双击则取消拖移
  if (dragTimer) { clearTimeout(dragTimer); dragTimer = null; return; }
  dragTimer = setTimeout(() => {
    dragTimer = null;
    import("@tauri-apps/api/window").then(m => m.getCurrentWindow().startDragging());
  }, 200);
}
onMounted(() => barRef.value?.addEventListener("mousedown", onMouseDown));
onUnmounted(() => barRef.value?.removeEventListener("mousedown", onMouseDown));

const isFullScreen = ref(false);
async function minimize() {
  window.electronAPI?.windowControl?.("minimize");
}
async function toggleMaximize() {
  const result = await window.electronAPI?.windowControl?.("maximize");
  if (result?.maximized !== undefined) isFullScreen.value = result.maximized;
}
async function close() {
  window.electronAPI?.windowControl?.("close");
}
// 初始化时查询窗口状态
(async () => {
  const state = await window.electronAPI?.windowControl?.("is_maximized");
  if (state) {
    isFullScreen.value = state.maximized || state.fullscreen || false;
  }
})();
window.electronAPI.on("close", () => {
  close();
});
</script>

<template>
  <div ref="barRef" class="TitleBarBox" @dblclick="toggleMaximize">
    <template v-if="isWinOrLinux">
      <MenuDropDown />
      <!-- <div class="title" @dblclick="toggleMaximize">
        {{ title }}
      </div> -->

      <TabBar />

      <div class="window-controls">
        <button class="window-control-btn" @click="minimize">
          <AppIcon name="min" />
        </button>
        <button class="window-control-btn" @click="toggleMaximize">
          <AppIcon :name="isFullScreen ? 'normal' : 'max'" />
        </button>
        <button class="window-control-btn close-btn" @click="close">
          <AppIcon name="close" />
        </button>
      </div>
    </template>
    <template v-else>
      <!-- macOS: 系统自带左上角交通灯按钮 -->
      <div style="width: 68px"></div>
      <TabBar />
      <div style="margin-right: 10px">
        <MenuDropDown />
      </div>
    </template>
  </div>
</template>

<style lang="less" scoped>
.TitleBarBox {
  height: 40px;
  background: var(--background-color-2);
  color: var(--text-color);
  display: flex;
  align-items: center;
  justify-content: space-between;

  .window-controls {
    display: flex;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;

    /* ✅ 控制按钮不能拖动 */
    .window-control-btn {
      display: flex;
      align-items: center;
      justify-content: center;
      cursor: pointer;
      font-size: 16px;
      color: var(--text-color-1);
      height: 40px;
      width: 40px;
      border: none;
      background: transparent;

      &:hover {
        background: var(--hover-color);
      }

      &.close-btn:hover {
        background: #ff5f56;
        color: white;
      }
    }
  }
}
</style>

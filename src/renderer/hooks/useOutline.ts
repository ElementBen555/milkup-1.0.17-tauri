import { ref } from "vue";
import emitter from "@/renderer/events";

const isShowOutline = ref(false);
function toggleShowOutline(status?: boolean | null) {
  const toggle = status !== null && status !== undefined;
  isShowOutline.value = toggle ? status : !isShowOutline.value;
}

// 全局存储最新大纲数据，避免组件挂载时机问题导致数据丢失
const outlineData = ref<{ id: string; level: number; text: string; pos: number }[]>([]);

// 注册监听（只一次，不依赖组件 onMounted）
emitter.on("outline:Update", (headings) => {
  outlineData.value = headings ?? [];
});

export default function useOutline() {
  return {
    outline: outlineData,
    isShowOutline,
  };
}
export { isShowOutline, toggleShowOutline };

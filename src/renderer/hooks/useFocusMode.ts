import { ref, watch } from "vue";

const isFocusMode = ref(false);

function toggleFocusMode() {
  isFocusMode.value = !isFocusMode.value;
  document.body.classList.toggle("focus-mode", isFocusMode.value);
}

export default function useFocusMode() {
  return { isFocusMode, toggleFocusMode };
}

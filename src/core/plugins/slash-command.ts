/**
 * 斜杠命令面板插件
 * 输入 `/` 触发，弹出快捷插入菜单
 */
import { Plugin, PluginKey } from "prosemirror-state";
import { Decoration, DecorationSet, type EditorView } from "prosemirror-view";

export const slashPluginKey = new PluginKey("slashCommand");

interface SlashCommand {
  label: string;
  icon: string;
  action: (view: EditorView, pos: number) => void;
}

const commands: SlashCommand[] = [
  { label: "标题 1", icon: "H1", action: setHeading(1) },
  { label: "标题 2", icon: "H2", action: setHeading(2) },
  { label: "标题 3", icon: "H3", action: setHeading(3) },
  { label: "代码块", icon: "</>", action: insertCodeBlock },
  { label: "引用", icon: "❝", action: wrapBlockquote },
  { label: "无序列表", icon: "•", action: wrapBulletList },
  { label: "有序列表", icon: "1.", action: wrapOrderedList },
  { label: "任务列表", icon: "☐", action: insertTaskList },
  { label: "表格", icon: "▦", action: insertTable },
  { label: "分割线", icon: "—", action: insertHr },
  { label: "数学公式", icon: "∑", action: insertMathBlock },
  { label: "图片", icon: "🖼", action: insertImageTrigger },
  { label: "目录", icon: "📑", action: insertToc },
];

function setHeading(level: number) {
  return (view: EditorView, pos: number) => {
    const { state, dispatch } = view;
    const tr = state.tr;
    const $pos = state.doc.resolve(pos);
    const blockStart = $pos.start($pos.depth);
    const blockEnd = $pos.end($pos.depth);
    const heading = state.schema.nodes.heading?.create({ level });
    if (heading && dispatch) {
      dispatch(tr.replaceWith(blockStart, blockEnd, heading));
      view.focus();
    }
  };
}

function insertCodeBlock(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const codeBlock = state.schema.nodes.code_block?.create({ language: "" });
  if (codeBlock && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), codeBlock);
    dispatch(tr);
    view.focus();
  }
}

function wrapBlockquote(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const blockquote = state.schema.nodes.blockquote?.create(
    null,
    state.schema.nodes.paragraph?.create() || undefined
  );
  if (blockquote && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), blockquote);
    dispatch(tr);
    view.focus();
  }
}

function wrapBulletList(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const list = state.schema.nodes.bullet_list?.create(
    null,
    state.schema.nodes.list_item?.create(null, state.schema.nodes.paragraph?.create()) || undefined
  );
  if (list && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), list);
    dispatch(tr);
    view.focus();
  }
}

function wrapOrderedList(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const list = state.schema.nodes.ordered_list?.create(
    null,
    state.schema.nodes.list_item?.create(null, state.schema.nodes.paragraph?.create()) || undefined
  );
  if (list && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), list);
    dispatch(tr);
    view.focus();
  }
}

function insertTaskList(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const item = state.schema.nodes.task_item?.create(
    { checked: false },
    state.schema.nodes.paragraph?.create() || null
  );
  const list = state.schema.nodes.task_list?.create(null, item || undefined);
  if (list && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), list);
    dispatch(tr);
    view.focus();
  }
}

function insertTable(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);

  const createCell = (content: string) =>
    state.schema.nodes.table_cell?.create(
      null,
      state.schema.nodes.paragraph?.create(null, content ? state.schema.text(content) : null) || undefined
    );

  const rows = [];
  for (let r = 0; r < 3; r++) {
    const cells = [];
    for (let c = 0; c < 3; c++) cells.push(createCell(r === 0 && c === 0 ? "Header" : ""));
    rows.push(state.schema.nodes.table_row?.create(null, cells.filter(Boolean) as any));
  }
  const table = state.schema.nodes.table?.create(null, rows.filter(Boolean) as any);
  if (table && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), table);
    dispatch(tr);
    view.focus();
  }
}

function insertHr(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const hr = state.schema.nodes.horizontal_rule?.create();
  if (hr && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), hr);
    const para = state.schema.nodes.paragraph?.create();
    if (para) tr.insert($pos.end($pos.depth) + hr.nodeSize, para);
    dispatch(tr);
    view.focus();
  }
}

function insertMathBlock(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const math = state.schema.nodes.math_block?.create(null, state.schema.text("x = y"));
  if (math && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), math);
    dispatch(tr);
    view.focus();
  }
}

function insertImageTrigger(view: EditorView, pos: number) {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = "image/*";
  input.onchange = () => {
    // Let the existing paste plugin handle it
    const file = input.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = () => {
        const img = view.state.schema.nodes.image?.create({
          src: reader.result as string,
          alt: file.name,
        });
        if (img) {
          view.dispatch(view.state.tr.replaceWith(pos, pos, img));
          view.focus();
        }
      };
      reader.readAsDataURL(file);
    }
  };
  input.click();
}

function insertToc(view: EditorView, pos: number) {
  const { state, dispatch } = view;
  const tr = state.tr;
  const $pos = state.doc.resolve(pos);
  const para = state.schema.nodes.paragraph?.create(
    null,
    state.schema.text("[TOC]")
  );
  if (para && dispatch) {
    tr.replaceWith($pos.start($pos.depth), $pos.end($pos.depth), para);
    dispatch(tr);
    view.focus();
  }
}

function showSlashMenu(view: EditorView, pos: number) {
  // Remove existing menu
  document.querySelector(".milkup-slash-menu")?.remove();

  const menu = document.createElement("div");
  menu.className = "milkup-slash-menu";
  menu.style.cssText = "position:fixed;z-index:99999;background:var(--background-color-1);border:1px solid var(--border-color-1);border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.2);max-height:320px;overflow-y:auto;min-width:200px;padding:4px;";

  commands.forEach((cmd, i) => {
    const item = document.createElement("div");
    item.className = "milkup-slash-item";
    item.style.cssText = "padding:8px 12px;cursor:pointer;display:flex;align-items:center;gap:10px;border-radius:4px;font-size:13px;color:var(--text-color-1);";
    item.innerHTML = `<span style="font-weight:600;width:24px;text-align:center">${cmd.icon}</span><span>${cmd.label}</span>`;
    item.addEventListener("mousedown", (e) => {
      e.preventDefault();
      cmd.action(view, pos);
      menu.remove();
    });
    menu.appendChild(item);
  });

  document.body.appendChild(menu);

  // Position near cursor
  const coords = view.coordsAtPos(pos);
  menu.style.left = `${coords.left}px`;
  menu.style.top = `${coords.bottom + 4}px`;

  // Close on outside click
  setTimeout(() => {
    document.addEventListener("click", function onClose(e) {
      if (!menu.contains(e.target as Node)) {
        menu.remove();
        document.removeEventListener("click", onClose);
      }
    });
  }, 0);

  // Keyboard navigation
  let selectedIdx = 0;
  const items = menu.querySelectorAll<HTMLElement>(".milkup-slash-item");
  const updateSelection = () => {
    items.forEach((el, i) => el.style.background = i === selectedIdx ? "var(--hover-color)" : "");
    items[selectedIdx]?.scrollIntoView({ block: "nearest" });
  };
  updateSelection();

  menu.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown") { e.preventDefault(); selectedIdx = Math.min(selectedIdx + 1, items.length - 1); updateSelection(); }
    if (e.key === "ArrowUp") { e.preventDefault(); selectedIdx = Math.max(selectedIdx - 1, 0); updateSelection(); }
    if (e.key === "Enter") { e.preventDefault(); commands[selectedIdx]?.action(view, pos); menu.remove(); }
    if (e.key === "Escape") { e.preventDefault(); menu.remove(); view.focus(); }
  });
}

export function createSlashPlugin(): Plugin {
  return new Plugin({
    key: slashPluginKey,
    props: {
      handleTextInput(view, from, to, text) {
        if (text === "/") {
          const $pos = view.state.doc.resolve(from);
          // Only at the start of a block (after the block's first position)
          const atBlockStart = $pos.parentOffset === 0;
          if (atBlockStart) {
            setTimeout(() => showSlashMenu(view, from + 1), 0);
          }
        }
        // Close menu on any other input
        document.querySelector(".milkup-slash-menu")?.remove();
        return false;
      },
      handleKeyDown(view, event) {
        if (event.key === "Escape") {
          const menu = document.querySelector(".milkup-slash-menu");
          if (menu) { menu.remove(); return true; }
        }
        return false;
      },
    },
  });
}

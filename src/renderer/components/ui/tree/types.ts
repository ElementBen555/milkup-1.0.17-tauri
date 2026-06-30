import type { HTMLAttributes } from "vue";

// 树节点接口
export interface TreeNode {
  name: string;
  path: string;
  isDirectory: boolean;
  mtime?: number;
  children?: TreeNode[];
}

// 树节点组件属性接口
export interface TreeNodeProps {
  node: TreeNode;
}

export interface TreeProps {
  nodes: TreeNode[] | null;
  currentNode: string | null;
  editingPath?: string | null;
  class?: HTMLAttributes["class"];
}

export interface TreeEmits {
  (e: "nodeClick", node: TreeNode): void;
  (e: "nodeContextMenu", node: TreeNode, event: MouseEvent): void;
  (e: "editConfirm", node: TreeNode, newName: string): void;
  (e: "editCancel", node: TreeNode): void;
}

import type { ComputedRef, Reactive } from "vue";
import type { TreeNode } from "./types";
import type { TransitionEffect } from "@/renderer/utils/heightTransition";
import { createContext } from "@/renderer/context/createContext";

export interface TreeContext {
  expandedNodes: Reactive<Set<string>>;
  currentNode: ComputedRef<string | null>;
  editingPath: ComputedRef<string | null>;
  transitionHooks: TransitionEffect;
  toggleNodeExpanded: (path: string) => void;
  clearExpandedNodes: () => void;
  handleNodeClick: (node: TreeNode) => void;
  handleNodeContextMenu: (node: TreeNode, event: MouseEvent) => void;
  handleEditConfirm: (node: TreeNode, newName: string) => void;
  handleEditCancel: (node: TreeNode) => void;
}

export const [useTreeContext, providerTreeContext] = createContext<TreeContext>("Tree");

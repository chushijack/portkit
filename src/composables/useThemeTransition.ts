/**
 * 文件名称：useThemeTransition.ts
 *
 * 文件功能：
 * 主题切换时播放圆形揭示动画，不支持时回退为颜色过渡。
 *
 * 主要职责：
 * - 优先使用 View Transitions
 * - 从点击位置向外铺开，避免硬切
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

import { nextTick } from "vue";

type ViewTransition = {
  ready: Promise<void>;
  finished: Promise<void>;
};

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function startViewTransition(update: () => void | Promise<void>): ViewTransition | null {
  const doc = document as Document & {
    startViewTransition?: (callback: () => void | Promise<void>) => ViewTransition;
  };
  if (typeof doc.startViewTransition !== "function") {
    return null;
  }
  return doc.startViewTransition(update);
}

function revealRadius(x: number, y: number): number {
  const right = window.innerWidth - x;
  const bottom = window.innerHeight - y;
  return Math.hypot(Math.max(x, right), Math.max(y, bottom));
}

/**
 * 在主题 class 切换前后插入过渡动画。
 */
export async function runThemeTransition(
  update: () => void | Promise<void>,
  origin?: { x: number; y: number },
): Promise<void> {
  if (prefersReducedMotion()) {
    await update();
    await nextTick();
    return;
  }

  const x = origin?.x ?? window.innerWidth / 2;
  const y = origin?.y ?? window.innerHeight / 2;

  const transition = startViewTransition(async () => {
    await update();
    await nextTick();
  });

  if (!transition) {
    await update();
    await nextTick();
    return;
  }

  document.documentElement.classList.add("theme-switching");
  try {
    await transition.ready;
    const radius = revealRadius(x, y);
    document.documentElement.animate(
      {
        clipPath: [`circle(0px at ${x}px ${y}px)`, `circle(${radius}px at ${x}px ${y}px)`],
      },
      {
        duration: 560,
        easing: "cubic-bezier(0.22, 1, 0.36, 1)",
        pseudoElement: "::view-transition-new(root)",
      },
    );
    await transition.finished;
  } catch {
    // 动画被中断时主题已经切完，忽略即可
  } finally {
    document.documentElement.classList.remove("theme-switching");
  }
}

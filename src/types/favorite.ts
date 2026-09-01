/**
 * 文件名称：favorite.ts
 *
 * 文件功能：
 * 定义收藏端口数据结构。
 *
 * 主要职责：
 * - 约束收藏列表字段
 *
 * 作者：Chushi Jack
 * 创建时间：2026-08-20
 */

export interface FavoritePort {
  port: number;
  protocol: string;
  note: string;
  createdAt: string;
}

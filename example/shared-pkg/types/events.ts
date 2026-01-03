/**
 * 玩家死亡事件
 * @param playerId 玩家ID
 * @param killerId 击杀者ID
 */
export class PlayerDeathEvent {
    public playerId: number;
    public killerId: number;
}

/**
 * 物品拾取事件
 * @param itemId 物品ID
 * @param count 数量
 */
export class ItemPickupEvent {
    public itemId: string;
    public count: number;
}

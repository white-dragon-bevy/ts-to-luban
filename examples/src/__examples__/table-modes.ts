/**
 * 不同 mode 的 table 示例
 */

import { LubanTable } from "../index";

/**
 * 排行榜条目 - list mode (纯数组，无索引)
 */
@LubanTable({ mode: "list", index: "rank" })
export class LeaderboardEntry {
    public rank: number;
    public playerId: number;
    public score: number;
}

/**
 * 游戏全局配置 - one mode (单条记录)
 */
@LubanTable({ mode: "one" })
export class GameConfig {
    public id: number;
    public maxPlayers: number;
    public gameVersion: string;
    public debugMode: boolean;
}

/**
 * 服务器设置 - singleton mode (单例)
 */
@LubanTable({ mode: "singleton"})
export class ServerSettings {
    public id: number;
    public serverName: string;
    public tickRate: number;
}

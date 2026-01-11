export interface EntityTrigger {}

export class DamageTrigger implements EntityTrigger {
    public damage: number;
    public radius: number;
}

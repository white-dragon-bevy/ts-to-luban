export type ObjectFactory<T> = () => T;

export class ComplexClass {
    public items: string[];
    public data: Map<string, number>;
    public trigger: ObjectFactory<Trigger>;
    public triggers: ObjectFactory<Trigger>[];
}

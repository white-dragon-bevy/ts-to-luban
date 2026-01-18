/**
 * Test for $type field filtering
 * $type is used for TypeScript discriminated unions but should not be exported to Luban
 */

export enum ShapeType {
    Circle = "circle",
    Rectangle = "rectangle",
}

/**
 * Shape info with discriminated union
 */
export interface ShapeInfo {
    /** Discriminator field - should NOT be exported to XML */
    $type: ShapeType;
    /** Width of the shape */
    width: number;
    /** Height of the shape */
    height: number;
}

/**
 * Circle shape
 */
export class CircleShape implements ShapeInfo {
    public $type: ShapeType = ShapeType.Circle;
    public width: number;
    public height: number;
    public radius: number;
}

/**
 * Rectangle shape
 */
export class RectangleShape implements ShapeInfo {
    public $type: ShapeType = ShapeType.Rectangle;
    public width: number;
    public height: number;
}

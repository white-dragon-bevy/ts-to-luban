// Test for private field handling
import { Required } from "../index";

/**
 * Test class with private fields
 * Private fields should be excluded from bean and creator generation
 */
export class TestPrivateFields {
    public id: number;

    @Required()
    public name: string;

    private internalId: string;

    private calculateId(): void {
        // This should be filtered out
    }
}

/**
 * Test interface - should return plain JSON object
 */
export interface ITestInterface {
    id: number;
    name: string;
}

/**
 * Test class with readonly fields
 * Readonly fields should use Writable<T> cast in creator
 */
export class TestReadonlyFields {
    public id: number;

    readonly createdAt: number;

    readonly updatedAt: number;
}

/**
 * Base class for all TypeScript classes that don't have explicit parent
 *
 * This is default parent for classes without extends or single implements.
 * Luban requires all parent references to exist in the schema.
 */
/**
 * @ignore
 */
export class TsClass {
    /**
     * Optional unique identifier
     */
    public id?: number;

    /**
     * Timestamp for tracking purposes
     */
    public createdAt?: number;
}

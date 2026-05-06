import { World } from '../types/world';
export declare class WorldModel {
    private static worlds;
    static create(data: {
        name: string;
        description: string;
    }): World;
    static findById(id: string): World | undefined;
    static findAll(): World[];
    static update(id: string, data: Partial<{
        name: string;
        description: string;
    }>): World | undefined;
    static delete(id: string): boolean;
}
//# sourceMappingURL=WorldModel.d.ts.map
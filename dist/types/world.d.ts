export interface World {
    id: string;
    name: string;
    description: string;
    createdAt: Date;
    updatedAt: Date;
}
export interface CreateWorldRequest {
    name: string;
    description: string;
}
export interface UpdateWorldRequest {
    name?: string;
    description?: string;
}
//# sourceMappingURL=world.d.ts.map
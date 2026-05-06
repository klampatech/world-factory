"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WorldModel = void 0;
const uuid_1 = require("uuid");
class WorldModel {
    static create(data) {
        const now = new Date();
        const world = {
            id: (0, uuid_1.v4)(),
            name: data.name,
            description: data.description,
            createdAt: now,
            updatedAt: now,
        };
        this.worlds.set(world.id, world);
        return world;
    }
    static findById(id) {
        return this.worlds.get(id);
    }
    static findAll() {
        return Array.from(this.worlds.values());
    }
    static update(id, data) {
        const world = this.worlds.get(id);
        if (!world)
            return undefined;
        if (data.name !== undefined)
            world.name = data.name;
        if (data.description !== undefined)
            world.description = data.description;
        world.updatedAt = new Date();
        this.worlds.set(id, world);
        return world;
    }
    static delete(id) {
        return this.worlds.delete(id);
    }
}
exports.WorldModel = WorldModel;
WorldModel.worlds = new Map();
//# sourceMappingURL=WorldModel.js.map
"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = require("express");
const WorldModel_1 = require("../models/WorldModel");
const router = (0, express_1.Router)();
// GET /api/v1/worlds - List all worlds
router.get('/', (_req, res) => {
    const worlds = WorldModel_1.WorldModel.findAll();
    res.json({ data: worlds });
});
// GET /api/v1/worlds/:id - Get a world by ID
router.get('/:id', (req, res) => {
    const world = WorldModel_1.WorldModel.findById(req.params.id);
    if (!world) {
        res.status(404).json({ error: { message: 'World not found', code: 'NOT_FOUND' } });
        return;
    }
    res.json({ data: world });
});
// POST /api/v1/worlds - Create a new world
router.post('/', (req, res) => {
    const { name, description } = req.body;
    if (!name || !description) {
        res.status(400).json({ error: { message: 'name and description are required', code: 'VALIDATION_ERROR' } });
        return;
    }
    const world = WorldModel_1.WorldModel.create({ name, description });
    res.status(201).json({ data: world });
});
// PUT /api/v1/worlds/:id - Update a world
router.put('/:id', (req, res) => {
    const world = WorldModel_1.WorldModel.update(req.params.id, req.body);
    if (!world) {
        res.status(404).json({ error: { message: 'World not found', code: 'NOT_FOUND' } });
        return;
    }
    res.json({ data: world });
});
// DELETE /api/v1/worlds/:id - Delete a world
router.delete('/:id', (req, res) => {
    const deleted = WorldModel_1.WorldModel.delete(req.params.id);
    if (!deleted) {
        res.status(404).json({ error: { message: 'World not found', code: 'NOT_FOUND' } });
        return;
    }
    res.status(204).send();
});
exports.default = router;
//# sourceMappingURL=worlds.js.map
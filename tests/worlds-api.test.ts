import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import request from 'supertest';
import express, { Express } from 'express';
import worldsRouter from '../src/routes/worlds';
import { WorldModel } from '../src/models/WorldModel';

describe('DELETE /api/v1/worlds/:id', () => {
  let app: Express;

  beforeEach(() => {
    app = express();
    app.use(express.json());
    app.use('/api/v1/worlds', worldsRouter);
    // Clear data between tests
    WorldModel.findAll().forEach(w => WorldModel.delete(w.id));
  });

  it('should return 204 on successful deletion', async () => {
    const world = WorldModel.create({ name: 'Test World', description: 'Test' });
    
    const res = await request(app)
      .delete(`/api/v1/worlds/${world.id}`)
      .expect(204);
    
    expect(res.body).toEqual({});
    expect(WorldModel.findById(world.id)).toBeUndefined();
  });

  it('should return 404 when world does not exist', async () => {
    const fakeId = '550e8400-e29b-41d4-a716-446655440000';
    
    const res = await request(app)
      .delete(`/api/v1/worlds/${fakeId}`)
      .expect(404);
    
    expect(res.body).toEqual({
      error: { message: 'World not found', code: 'NOT_FOUND' }
    });
  });

  it('should return 400 for invalid UUID format', async () => {
    const res = await request(app)
      .delete('/api/v1/worlds/invalid-id')
      .expect(400);
    
    expect(res.body).toEqual({
      error: { message: 'Invalid UUID format', code: 'VALIDATION_ERROR' }
    });
  });

  it('should return 400 for malformed UUID (missing segments)', async () => {
    const res = await request(app)
      .delete('/api/v1/worlds/12345678-1234')
      .expect(400);
    
    expect(res.body.error.code).toBe('VALIDATION_ERROR');
  });
});
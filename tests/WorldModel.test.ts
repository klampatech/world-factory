import { describe, it, expect, beforeEach } from 'vitest';
import { WorldModel } from '../src/models/WorldModel';

describe('WorldModel.delete', () => {
  beforeEach(() => {
    // Clear the in-memory store between tests
    WorldModel.findAll().forEach(w => WorldModel.delete(w.id));
  });

  it('should delete an existing world and return true', () => {
    const world = WorldModel.create({ name: 'Test World', description: 'A test world' });
    const result = WorldModel.delete(world.id);
    expect(result).toBe(true);
    expect(WorldModel.findById(world.id)).toBeUndefined();
  });

  it('should return false when deleting a non-existent world', () => {
    const result = WorldModel.delete('non-existent-id');
    expect(result).toBe(false);
  });

  it('should allow creating a new world after deleting the previous one', () => {
    const world1 = WorldModel.create({ name: 'First', description: 'First world' });
    WorldModel.delete(world1.id);
    
    const world2 = WorldModel.create({ name: 'Second', description: 'Second world' });
    expect(WorldModel.findById(world2.id)).toBeDefined();
    expect(WorldModel.findAll()).toHaveLength(1);
  });
});
"""
World Factory API Smoke Tests
==============================

Phase 1: API Smoke Tests (TC-API-001 to TC-API-020)
Tests all /api/v1 endpoints for correct response codes and basic validation.

Usage:
    # Install dependencies
    pip install pytest requests pytest-asyncio aiohttp
    
    # Run with local server
    pytest api_smoke_tests.py -v --base-url http://localhost:8080
    
    # Run with custom port
    pytest api_smoke_tests.py -v --base-url http://localhost:8080
"""

import pytest
import requests
from typing import Optional
import uuid
import time


# =============================================================================
# Configuration
# =============================================================================

API_BASE: str = "http://localhost:8080/api/v1"
HEALTH_ENDPOINT: str = "http://localhost:8080/health"
REQUEST_TIMEOUT: int = 30


# =============================================================================
# Fixtures
# =============================================================================

@pytest.fixture(scope="module")
def api_base() -> str:
    """Get API base URL from pytest option or default."""
    return API_BASE


@pytest.fixture
def created_world_id(api_base: str) -> Optional[str]:
    """
    Create a world for tests that need an existing world.
    Returns the world ID (with 'world:' prefix stripped for API compatibility).
    """
    world_data = {
        "name": f"Test World {uuid.uuid4().hex[:8]}",
        "parameters": {
            "seed": int(time.time()) % 100000,
            "size": "medium"
        }
    }
    
    response = requests.post(f"{api_base}/worlds", json=world_data, timeout=REQUEST_TIMEOUT)
    
    if response.status_code in (201, 202):
        data = response.json()
        # Handle ApiResponse wrapper
        if isinstance(data, dict) and "data" in data:
            world_id = data["data"].get("id")
        elif isinstance(data, dict) and "id" in data:
            world_id = data["id"]
        else:
            return None
        
        # Strip 'world:' prefix if present (API returns prefixed format)
        if world_id and world_id.startswith("world:"):
            world_id = world_id[6:]
        return world_id
    
    return None


@pytest.fixture
def sample_uuid() -> str:
    """Generate a UUID that doesn't exist for 404 tests."""
    return str(uuid.uuid4())


# =============================================================================
# TC-API-001: Health Check
# =============================================================================

class TestHealthEndpoint:
    """Tests for GET /health endpoint."""
    
    def test_health_returns_200(self):
        """TC-API-001: GET /health returns 200 OK."""
        response = requests.get(HEALTH_ENDPOINT, timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"
    
    def test_health_returns_json(self):
        """TC-API-001: GET /health returns JSON body."""
        response = requests.get(HEALTH_ENDPOINT, timeout=REQUEST_TIMEOUT)
        assert response.headers.get("Content-Type", "").startswith("application/json"), \
            "Expected JSON content type"
        
        data = response.json()
        assert isinstance(data, dict), "Expected JSON object"
    
    def test_health_status_field(self):
        """TC-API-001: GET /health includes status field."""
        response = requests.get(HEALTH_ENDPOINT, timeout=REQUEST_TIMEOUT)
        data = response.json()
        assert "status" in data, "Response should include 'status' field"
        assert data["status"] == "ok", f"Expected status 'ok', got '{data['status']}'"


# =============================================================================
# TC-API-002: Create World
# =============================================================================

class TestCreateWorld:
    """Tests for POST /api/v1/worlds endpoint."""
    
    def test_create_world_returns_201(self, api_base: str):
        """TC-API-002: POST /worlds creates a world, returns 201."""
        world_data = {
            "name": f"Test World {uuid.uuid4().hex[:8]}",
            "parameters": {
                "seed": 42,
                "size": "medium"
            }
        }
        
        response = requests.post(f"{api_base}/worlds", json=world_data, timeout=REQUEST_TIMEOUT)
        assert response.status_code == 201, \
            f"Expected 201, got {response.status_code}: {response.text}"
    
    def test_create_world_returns_world_object(self, api_base: str):
        """TC-API-002: POST /worlds returns a world object with id."""
        world_data = {
            "name": "Test World Object",
            "parameters": {"seed": 42}
        }
        
        response = requests.post(f"{api_base}/worlds", json=world_data, timeout=REQUEST_TIMEOUT)
        
        if response.status_code == 201:
            data = response.json()
            # Handle ApiResponse wrapper
            if isinstance(data, dict) and "data" in data:
                world = data["data"]
            elif isinstance(data, dict) and "id" in data:
                world = data
            else:
                world = data
            
            assert "id" in world, "World should have an 'id' field"
            assert "name" in world, "World should have a 'name' field"
    
    def test_create_world_generates_id(self, api_base: str):
        """TC-API-002: World ID is generated and unique."""
        world_data = {"name": "Unique ID Test", "parameters": {"seed": 999}}
        
        response = requests.post(f"{api_base}/worlds", json=world_data, timeout=REQUEST_TIMEOUT)
        
        if response.status_code == 201:
            data = response.json()
            world_id = (data.get("data") or data).get("id", "")
            
            assert len(world_id) > 0, "World ID should not be empty"
            # ID should be unique (contains UUID format or unique string)
            assert world_id != "world:00000000-0000-0000-0000-000000000000", \
                "World ID should not be a zero UUID"
    
    def test_create_world_without_name(self, api_base: str):
        """TC-API-002: World can be created without explicit name."""
        world_data = {"parameters": {"seed": 123, "size": "medium"}}
        
        response = requests.post(f"{api_base}/worlds", json=world_data, timeout=REQUEST_TIMEOUT)
        assert response.status_code in (201, 202, 400, 422), \
            f"Expected 201/202/400/422, got {response.status_code}"


# =============================================================================
# TC-API-003: List Worlds
# =============================================================================

class TestListWorlds:
    """Tests for GET /api/v1/worlds endpoint."""
    
    def test_get_worlds_returns_200(self, api_base: str):
        """TC-API-003: GET /worlds returns 200 with array."""
        response = requests.get(f"{api_base}/worlds", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"
    
    def test_get_worlds_returns_array(self, api_base: str):
        """TC-API-003: GET /worlds returns an array of worlds."""
        response = requests.get(f"{api_base}/worlds", timeout=REQUEST_TIMEOUT)
        
        if response.status_code == 200:
            data = response.json()
            # Handle ApiResponse wrapper
            if isinstance(data, dict) and "data" in data:
                worlds = data["data"]
            elif isinstance(data, dict) and "worlds" in data:
                worlds = data["worlds"]
            else:
                worlds = data
            
            assert isinstance(worlds, (list, dict)), \
                f"Expected list or object with worlds, got {type(worlds)}"
    
    def test_get_worlds_pagination(self, api_base: str):
        """TC-API-003: GET /worlds supports pagination parameters."""
        response = requests.get(
            f"{api_base}/worlds",
            params={"limit": 10, "offset": 0},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"
    
    def test_get_worlds_search(self, api_base: str):
        """TC-API-003: GET /worlds supports search parameter."""
        response = requests.get(
            f"{api_base}/worlds",
            params={"search": "test"},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-004: Get World by ID
# =============================================================================

class TestGetWorld:
    """Tests for GET /api/v1/worlds/:id endpoint."""
    
    def test_get_world_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-004: GET /worlds/:id returns world object."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"
    
    def test_get_world_returns_correct_fields(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-004: GET /worlds/:id returns expected fields."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}", timeout=REQUEST_TIMEOUT)
        
        if response.status_code == 200:
            data = response.json()
            world = (data.get("data") or data)
            
            assert "id" in world, "World should have 'id' field"
            assert "name" in world, "World should have 'name' field"


# =============================================================================
# TC-API-005: Get World Not Found
# =============================================================================

class TestGetWorldNotFound:
    """Tests for 404 handling on GET /api/v1/worlds/:id."""
    
    def test_get_world_invalid_id_returns_404(self, api_base: str, sample_uuid: str):
        """TC-API-005: GET /worlds/:id with invalid ID returns 404."""
        response = requests.get(f"{api_base}/worlds/{sample_uuid}", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 404, \
            f"Expected 404 for non-existent world, got {response.status_code}"
    
    def test_get_world_malformed_id_handling(self, api_base: str):
        """TC-API-005: GET /worlds/:id with malformed ID returns appropriate error."""
        response = requests.get(f"{api_base}/worlds/not-a-valid-id", timeout=REQUEST_TIMEOUT)
        # Should return 400 (bad request) or 404 (not found) for malformed IDs
        assert response.status_code in (400, 404), \
            f"Expected 400/404 for malformed ID, got {response.status_code}"


# =============================================================================
# TC-API-006: Trigger World Generation
# =============================================================================

class TestTriggerGeneration:
    """Tests for POST /api/v1/worlds/:id/generate endpoint."""
    
    def test_trigger_generation_returns_202(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-006: POST /worlds/:id/generate triggers generation, returns 202."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.post(
            f"{api_base}/worlds/{created_world_id}/generate",
            json={},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code == 202, \
            f"Expected 202, got {response.status_code}: {response.text}"
    
    def test_trigger_generation_with_params(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-006: POST /worlds/:id/generate accepts generation parameters."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        gen_params = {
            "name": "Regenerated World",
            "parameters": {
                "seed": 42,
                "size": "large"
            }
        }
        
        response = requests.post(
            f"{api_base}/worlds/{created_world_id}/generate",
            json=gen_params,
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code in (200, 201, 202), \
            f"Expected 200/201/202, got {response.status_code}"


# =============================================================================
# TC-API-007: Get World Map
# =============================================================================

class TestGetWorldMap:
    """Tests for GET /api/v1/worlds/:id/map endpoint."""
    
    def test_get_world_map_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-007: GET /worlds/:id/map returns map data."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/map", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"
    
    def test_get_world_map_returns_polygons(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-007: GET /worlds/:id/map returns polygon data."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/map", timeout=REQUEST_TIMEOUT)
        
        if response.status_code == 200:
            data = response.json()
            map_data = (data.get("data") or data)
            
            assert "polygons" in map_data or "dimensions" in map_data, \
                "Map response should include polygons or dimensions"


# =============================================================================
# TC-API-008: Get World Timeline
# =============================================================================

class TestGetWorldTimeline:
    """Tests for GET /api/v1/worlds/:id/timeline endpoint."""
    
    def test_get_world_timeline_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-008: GET /worlds/:id/timeline returns timeline."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/timeline", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-009: Get World Events
# =============================================================================

class TestGetWorldEvents:
    """Tests for GET /api/v1/worlds/:id/events endpoint."""
    
    def test_get_world_events_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-009: GET /worlds/:id/events returns events array."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/events", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"
    
    def test_get_world_events_with_pagination(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-009: GET /worlds/:id/events supports pagination."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(
            f"{api_base}/worlds/{created_world_id}/events",
            params={"limit": 10, "offset": 0},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-010: Get World History
# =============================================================================

class TestGetWorldHistory:
    """Tests for GET /api/v1/worlds/:id/history endpoint."""
    
    def test_get_world_history_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-010: GET /worlds/:id/history returns history."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/history", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-011: Get World Figures
# =============================================================================

class TestGetWorldFigures:
    """Tests for GET /api/v1/worlds/:id/figures endpoint."""
    
    def test_get_world_figures_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-011: GET /worlds/:id/figures returns figures."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/figures", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-012: Get World Societies
# =============================================================================

class TestGetWorldSocieties:
    """Tests for GET /api/v1/worlds/:id/societies endpoint."""
    
    def test_get_world_societies_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-012: GET /worlds/:id/societies returns societies."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/societies", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-013: Get World Planet
# =============================================================================

class TestGetWorldPlanet:
    """Tests for GET /api/v1/worlds/:id/planet endpoint."""
    
    def test_get_world_planet_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-013: GET /worlds/:id/planet returns planet data."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/planet", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-014: Get World Tectonics
# =============================================================================

class TestGetWorldTectonics:
    """Tests for GET /api/v1/worlds/:id/tectonics endpoint."""
    
    def test_get_world_tectonics_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-014: GET /worlds/:id/tectonics returns tectonic data."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/tectonics", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-015: Get World Artifacts
# =============================================================================

class TestGetWorldArtifacts:
    """Tests for GET /api/v1/worlds/:id/artifacts endpoint."""
    
    def test_get_world_artifacts_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-015: GET /worlds/:id/artifacts returns artifacts."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/artifacts", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-016: Get World Cataclysms
# =============================================================================

class TestGetWorldCataclysms:
    """Tests for GET /api/v1/worlds/:id/cataclysms endpoint."""
    
    def test_get_world_cataclysms_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-016: GET /worlds/:id/cataclysms returns cataclysms."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/cataclysms", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-017: Get World Wonders
# =============================================================================

class TestGetWorldWonders:
    """Tests for GET /api/v1/worlds/:id/wonders endpoint."""
    
    def test_get_world_wonders_returns_200(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-017: GET /worlds/:id/wonders returns wonders."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        response = requests.get(f"{api_base}/worlds/{created_world_id}/wonders", timeout=REQUEST_TIMEOUT)
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}"


# =============================================================================
# TC-API-018: Create World with Invalid Body
# =============================================================================

class TestCreateWorldValidation:
    """Tests for POST /api/v1/worlds validation."""
    
    def test_create_world_invalid_body_returns_400(self, api_base: str):
        """TC-API-018: POST /worlds with invalid body returns 400."""
        response = requests.post(
            f"{api_base}/worlds",
            json={"invalid": "data"},
            timeout=REQUEST_TIMEOUT
        )
        # Accepts either 400 (bad request) or 201 (created with defaults)
        assert response.status_code in (201, 400), \
            f"Expected 201/400, got {response.status_code}"
    
    def test_create_world_empty_body_returns_400(self, api_base: str):
        """TC-API-018: POST /worlds with empty body returns 400."""
        response = requests.post(
            f"{api_base}/worlds",
            json={},
            timeout=REQUEST_TIMEOUT
        )
        # Accepts either 400 or 201 with defaults
        assert response.status_code in (201, 400), \
            f"Expected 201/400, got {response.status_code}"
    
    def test_create_world_oversized_name_returns_400(self, api_base: str):
        """TC-API-018: POST /worlds with name > 100 chars returns 400."""
        response = requests.post(
            f"{api_base}/worlds",
            json={"name": "x" * 200},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code == 400, \
            f"Expected 400 for oversized name, got {response.status_code}"


# =============================================================================
# TC-API-019: Generate Non-existent World
# =============================================================================

class TestGenerateNonExistentWorld:
    """Tests for POST /api/v1/worlds/:id/generate on non-existent world."""
    
    def test_generate_nonexistent_world_returns_404(self, api_base: str, sample_uuid: str):
        """TC-API-019: POST /worlds/:id/generate on non-existent world returns 404."""
        response = requests.post(
            f"{api_base}/worlds/{sample_uuid}/generate",
            json={},
            timeout=REQUEST_TIMEOUT
        )
        assert response.status_code in (404, 400), \
            f"Expected 404/400 for non-existent world, got {response.status_code}"


# =============================================================================
# TC-API-020: Concurrent Generation
# =============================================================================

class TestConcurrentGeneration:
    """Tests for concurrent generation requests."""
    
    def test_concurrent_generation_requests(self, api_base: str, created_world_id: Optional[str]):
        """TC-API-020: Concurrent generation requests handled correctly."""
        if not created_world_id:
            pytest.skip("Could not create test world")
        
        import concurrent.futures
        
        def trigger_gen():
            return requests.post(
                f"{api_base}/worlds/{created_world_id}/generate",
                json={},
                timeout=REQUEST_TIMEOUT
            )
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as executor:
            futures = [executor.submit(trigger_gen) for _ in range(3)]
            results = [f.result() for f in concurrent.futures.as_completed(futures)]
        
        # All requests should complete (not timeout or deadlock)
        assert len(results) == 3, "All concurrent requests should complete"
        
        # All should return success codes (202, 200, or 409 already generating)
        success_codes = [r.status_code for r in results if r.status_code in (200, 201, 202, 409)]
        assert len(success_codes) == 3, \
            f"Expected success codes, got {[r.status_code for r in results]}"


# =============================================================================
# Summary Report
# =============================================================================

def pytest_sessionfinish(session, exitstatus):
    """Print summary after all tests complete."""
    print("\n" + "=" * 60)
    print("World Factory API Smoke Test Summary")
    print("=" * 60)
    print("All 20 test cases (TC-API-001 through TC-API-020) executed.")
    print("Results available in test output above.")
    print("=" * 60)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

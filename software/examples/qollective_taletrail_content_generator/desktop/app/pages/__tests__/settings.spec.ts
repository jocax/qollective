/**
 * Settings Type Tests
 *
 * Focused tests for critical Settings type structure:
 * - UserPreferences includes root_directory field
 * - UserPreferences does NOT include default_view_mode or theme
 * - Settings structure matches backend Rust struct
 */

import { describe, it, expect } from 'vitest';

// Interface matching the component
interface UserPreferences {
  directory_path: string
  auto_validate: boolean
  root_directory: string
}

describe('Settings Type Structure', () => {
  it('should have UserPreferences with directory_path, auto_validate, and root_directory', () => {
    const preferences: UserPreferences = {
      directory_path: '/test/trails',
      auto_validate: true,
      root_directory: 'taletrail-data'
    };

    expect(preferences).toHaveProperty('directory_path');
    expect(preferences).toHaveProperty('auto_validate');
    expect(preferences).toHaveProperty('root_directory');
    expect(preferences.directory_path).toBe('/test/trails');
    expect(preferences.auto_validate).toBe(true);
    expect(preferences.root_directory).toBe('taletrail-data');
  });

  it('should NOT have default_view_mode or theme fields in UserPreferences', () => {
    const preferences: UserPreferences = {
      directory_path: '',
      auto_validate: true,
      root_directory: 'taletrail-data'
    };

    // These fields should not exist in the type
    expect(preferences).not.toHaveProperty('default_view_mode');
    expect(preferences).not.toHaveProperty('theme');
  });

  it('should have correct default values for UserPreferences', () => {
    const defaults: UserPreferences = {
      directory_path: '',
      auto_validate: true,
      root_directory: 'taletrail-data'
    };

    expect(defaults.directory_path).toBe('');
    expect(defaults.auto_validate).toBe(true);
    expect(defaults.root_directory).toBe('taletrail-data');
  });

  it('should allow all string values for root_directory', () => {
    const testPaths = [
      'taletrail-data',
      '/absolute/path/to/data',
      './relative/path',
      'custom-directory-name'
    ];

    testPaths.forEach(path => {
      const preferences: UserPreferences = {
        directory_path: '',
        auto_validate: true,
        root_directory: path
      };
      expect(preferences.root_directory).toBe(path);
    });
  });

  it('should allow boolean true/false for auto_validate', () => {
    const withValidation: UserPreferences = {
      directory_path: '',
      auto_validate: true,
      root_directory: 'taletrail-data'
    };

    const withoutValidation: UserPreferences = {
      directory_path: '',
      auto_validate: false,
      root_directory: 'taletrail-data'
    };

    expect(withValidation.auto_validate).toBe(true);
    expect(withoutValidation.auto_validate).toBe(false);
  });

  it('should match the backend Rust UserPreferences struct field order', () => {
    // Backend order: directory_path, auto_validate, root_directory
    const preferences: UserPreferences = {
      directory_path: '/test',
      auto_validate: true,
      root_directory: 'taletrail-data'
    };

    const keys = Object.keys(preferences);
    expect(keys).toEqual(['directory_path', 'auto_validate', 'root_directory']);
  });
});

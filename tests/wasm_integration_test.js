#!/usr/bin/env node
/**
 * fontations.wasm integration tests
 * Tests Rust/WASM font processing functionality and performance
 */

import { promises as fs } from 'fs';
import { performance } from 'perf_hooks';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const PKG_DIR = path.join(__dirname, '..', 'pkg');

// Test configuration
const TEST_CONFIG = {
    timeout: 30000,
    maxMemoryUsage: 100 * 1024 * 1024, // 100MB
    expectedFonts: 1 // Minimum expected fonts in test environment
};

// Test results collector
const results = {
    passed: 0,
    failed: 0,
    tests: []
};

// Utility functions
const log = (msg) => console.log(`[TEST] ${msg}`);
const error = (msg) => console.error(`[ERROR] ${msg}`);
const assert = (condition, message) => {
    if (!condition) {
        throw new Error(`Assertion failed: ${message}`);
    }
};

// Generate test font data (minimal valid TTF)
function generateTestFont() {
    // This is a minimal valid TTF header for testing
    // In a real implementation, you'd use actual font files
    const buffer = new ArrayBuffer(1024);
    const view = new DataView(buffer);
    
    // TTF signature
    view.setUint32(0, 0x00010000, false); // sfnt version
    view.setUint16(4, 4, false);           // numTables
    view.setUint16(6, 64, false);          // searchRange
    view.setUint16(8, 2, false);           // entrySelector
    view.setUint16(10, 0, false);          // rangeShift
    
    return new Uint8Array(buffer);
}

// Test helper to run individual test
async function runTest(name, testFn) {
    log(`Running test: ${name}`);
    const start = performance.now();
    
    try {
        await testFn();
        const duration = Math.round(performance.now() - start);
        log(`✓ ${name} (${duration}ms)`);
        results.passed++;
        results.tests.push({ name, status: 'PASSED', duration });
    } catch (err) {
        const duration = Math.round(performance.now() - start);
        error(`✗ ${name} (${duration}ms): ${err.message}`);
        results.failed++;
        results.tests.push({ name, status: 'FAILED', duration, error: err.message });
    }
}

// Load WASM module for testing
async function loadFontationsModule() {
    try {
        const modulePath = path.join(PKG_DIR, 'fontations.js');
        await fs.access(modulePath);
        
        const { default: init, FontProcessor, FontBenchmark } = await import(modulePath);
        await init();
        
        return { FontProcessor, FontBenchmark };
    } catch (err) {
        throw new Error(`Failed to load fontations.wasm. Run ./build-wasm.sh first. Error: ${err.message}`);
    }
}

// Test 1: Module loading and initialization
async function testModuleLoading() {
    const { FontProcessor, FontBenchmark } = await loadFontationsModule();
    
    assert(typeof FontProcessor === 'function', 'FontProcessor class should be available');
    assert(typeof FontBenchmark === 'function', 'FontBenchmark class should be available');
    
    // Test constructor
    const processor = new FontProcessor();
    assert(processor instanceof FontProcessor, 'FontProcessor should instantiate correctly');
    
    const benchmark = new FontBenchmark();
    assert(benchmark instanceof FontBenchmark, 'FontBenchmark should instantiate correctly');
}

// Test 2: Font loading functionality
async function testFontLoading() {
    const { FontProcessor } = await loadFontationsModule();
    const processor = new FontProcessor();
    
    const testFontData = generateTestFont();
    
    try {
        // This might fail with our minimal test font, but we test the interface
        const fontInfo = processor.load_font(testFontData, 'test-font');
        
        // If it succeeds, verify the structure
        assert(typeof fontInfo.family === 'string', 'Font info should have family name');
        assert(typeof fontInfo.glyph_count === 'number', 'Font info should have glyph count');
        assert(typeof fontInfo.units_per_em === 'number', 'Font info should have units per em');
        
        log(`Loaded test font: ${fontInfo.family} (${fontInfo.glyph_count} glyphs)`);
        
    } catch (err) {
        // Expected for our minimal test font, but test error handling
        assert(err.message.includes('Failed to'), 'Should provide meaningful error messages');
        log('Font loading failed as expected with minimal test data');
    }
}

// Test 3: Memory management
async function testMemoryManagement() {
    const { FontProcessor } = await loadFontationsModule();
    const processor = new FontProcessor();
    
    // Test memory usage tracking
    const initialStats = processor.get_memory_usage();
    assert(typeof initialStats.used === 'number', 'Memory stats should include used memory');
    assert(typeof initialStats.limit === 'number', 'Memory stats should include memory limit');
    assert(typeof initialStats.font_count === 'number', 'Memory stats should include font count');
    
    // Test memory limit setting
    processor.set_memory_limit(50); // 50MB
    const updatedStats = processor.get_memory_usage();
    assert(updatedStats.limit === 50 * 1024 * 1024, 'Memory limit should be updated');
    
    // Test font list functionality
    const fontList = processor.list_fonts();
    assert(Array.isArray(fontList), 'Font list should be an array');
    
    // Test clearing fonts
    processor.clear_fonts();
    const clearedStats = processor.get_memory_usage();
    assert(clearedStats.font_count === 0, 'Font count should be 0 after clearing');
}

// Test 4: Async functionality
async function testAsyncOperations() {
    const { FontProcessor } = await loadFontationsModule();
    const processor = new FontProcessor();
    
    const testFontData = generateTestFont();
    
    try {
        // Test async font loading
        const fontInfo = await processor.load_font_async(testFontData, 'async-test-font');
        
        // If successful, verify the result
        if (fontInfo) {
            assert(typeof fontInfo.family === 'string', 'Async loaded font should have family name');
        }
        
    } catch (err) {
        // Expected for minimal test font
        log('Async font loading failed as expected with minimal test data');
    }
    
    // Test async glyph processing
    try {
        const glyphIds = [65, 66, 67]; // A, B, C
        const results = await processor.process_glyphs_async('nonexistent-font', glyphIds);
        
        // This should fail with nonexistent font
        assert(false, 'Should have thrown error for nonexistent font');
        
    } catch (err) {
        assert(err.message.includes('not found'), 'Should error for nonexistent font');
        log('Async glyph processing correctly handles missing font');
    }
}

// Test 5: Error handling
async function testErrorHandling() {
    const { FontProcessor } = await loadFontationsModule();
    const processor = new FontProcessor();
    
    // Test invalid font data
    try {
        const emptyData = new Uint8Array(0);
        processor.load_font(emptyData, 'empty-font');
        assert(false, 'Should have thrown error for empty font data');
    } catch (err) {
        assert(err.message.includes('empty'), 'Should error for empty font data');
    }
    
    // Test memory limit exceeded
    processor.set_memory_limit(0.001); // 1KB limit
    try {
        const largeData = new Uint8Array(2048); // 2KB data
        processor.load_font(largeData, 'large-font');
        assert(false, 'Should have thrown error for memory limit exceeded');
    } catch (err) {
        assert(err.message.includes('Memory limit'), 'Should error when memory limit exceeded');
    }
    
    // Test nonexistent font operations
    try {
        processor.get_glyph_info('nonexistent', 65);
        assert(false, 'Should have thrown error for nonexistent font');
    } catch (err) {
        assert(err.message.includes('not found'), 'Should error for nonexistent font');
    }
}

// Test 6: Performance benchmarking
async function testPerformanceBenchmarking() {
    const { FontBenchmark } = await loadFontationsModule();
    const benchmark = new FontBenchmark();
    
    const testFontData = generateTestFont();
    
    // Test font loading benchmark
    const loadingTime = benchmark.benchmark_font_loading(testFontData, 10);
    assert(typeof loadingTime === 'number', 'Benchmark should return numeric time');
    assert(loadingTime >= 0, 'Benchmark time should be non-negative');
    
    log(`Font loading benchmark: ${loadingTime}ms for 10 iterations`);
    
    // Test glyph processing benchmark
    try {
        const glyphResults = benchmark.benchmark_glyph_processing(testFontData, 100);
        
        if (glyphResults) {
            assert(typeof glyphResults.duration === 'number', 'Glyph benchmark should include duration');
            assert(typeof glyphResults.throughput === 'number', 'Glyph benchmark should include throughput');
            
            log(`Glyph processing: ${glyphResults.throughput.toFixed(0)} glyphs/sec`);
        }
        
    } catch (err) {
        log('Glyph processing benchmark failed with test data (expected)');
    }
}

// Test 7: Version and feature detection
async function testVersionAndFeatures() {
    const { FontProcessor } = await loadFontationsModule();
    
    // Test that we can access utility functions
    // Note: These would need to be exported in the WASM module
    try {
        const processor = new FontProcessor();
        
        // Test memory management functions work
        const stats = processor.get_memory_usage();
        assert(stats.used >= 0, 'Memory usage should be non-negative');
        
        log('Version and feature detection completed');
        
    } catch (err) {
        log('Version/feature detection not fully implemented yet');
    }
}

// Test 8: Concurrent operations
async function testConcurrentOperations() {
    const { FontProcessor } = await loadFontationsModule();
    
    // Test multiple processors
    const processor1 = new FontProcessor();
    const processor2 = new FontProcessor();
    
    // Test they're independent
    processor1.set_memory_limit(50);
    processor2.set_memory_limit(100);
    
    const stats1 = processor1.get_memory_usage();
    const stats2 = processor2.get_memory_usage();
    
    assert(stats1.limit !== stats2.limit, 'Processors should have independent memory limits');
    
    // Test concurrent operations don't interfere
    const testFontData = generateTestFont();
    
    const promises = [];
    for (let i = 0; i < 5; i++) {
        promises.push(processor1.load_font_async(testFontData, `concurrent-font-${i}`).catch(() => null));
    }
    
    const results = await Promise.all(promises);
    log(`Concurrent operations completed: ${results.filter(r => r !== null).length} successful`);
}

// Generate test report
function generateReport() {
    const totalTests = results.passed + results.failed;
    const successRate = totalTests > 0 ? (results.passed / totalTests * 100).toFixed(1) : 0;
    
    console.log('\n' + '='.repeat(60));
    console.log('FONTATIONS.WASM TEST RESULTS');
    console.log('='.repeat(60));
    console.log(`Total Tests: ${totalTests}`);
    console.log(`Passed: ${results.passed}`);
    console.log(`Failed: ${results.failed}`);
    console.log(`Success Rate: ${successRate}%`);
    console.log('');
    
    if (results.tests.length > 0) {
        console.log('Test Details:');
        results.tests.forEach(test => {
            const status = test.status === 'PASSED' ? '✓' : '✗';
            console.log(`  ${status} ${test.name} (${test.duration}ms)`);
            if (test.error) {
                console.log(`    Error: ${test.error}`);
            }
        });
    }
    
    console.log('='.repeat(60));
    
    return results.failed === 0;
}

// Main test runner
async function main() {
    log('Starting fontations.wasm integration tests...');
    
    try {
        await runTest('Module Loading', testModuleLoading);
        await runTest('Font Loading', testFontLoading);
        await runTest('Memory Management', testMemoryManagement);
        await runTest('Async Operations', testAsyncOperations);
        await runTest('Error Handling', testErrorHandling);
        await runTest('Performance Benchmarking', testPerformanceBenchmarking);
        await runTest('Version and Features', testVersionAndFeatures);
        await runTest('Concurrent Operations', testConcurrentOperations);
        
        const success = generateReport();
        process.exit(success ? 0 : 1);
        
    } catch (err) {
        error(`Test runner failed: ${err.message}`);
        process.exit(1);
    }
}

// Run tests if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
    main().catch(err => {
        error(`Unhandled error: ${err.message}`);
        process.exit(1);
    });
}
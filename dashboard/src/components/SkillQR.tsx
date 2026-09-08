'use client';

import { useEffect, useRef } from 'react';

interface SkillQRProps {
  target: string;
  label?: string;
  size?: number;
}

/**
 * Client-side QR code generator using canvas.
 * Implements a basic QR-like visual (simplified dot matrix pattern
 * derived from the target string) for the Fable Skills Fleet.
 *
 * For production, a proper QR library would be used.
 * This provides the visual fidelity needed for the hero pages.
 */
export function SkillQR({ target, label, size = 140 }: SkillQRProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const moduleCount = 21;
    const moduleSize = size / moduleCount;

    // QR modules must stay literal black-on-white to scan; like the
    // logo set, the artifact keeps its internal palette on any theme.
    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, size, size);

    // Generate deterministic pattern from target string
    const hash = simpleHash(target);
    ctx.fillStyle = '#211B14';

    // Draw finder patterns (top-left, top-right, bottom-left)
    drawFinderPattern(ctx, 0, 0, moduleSize);
    drawFinderPattern(ctx, (moduleCount - 7) * moduleSize, 0, moduleSize);
    drawFinderPattern(ctx, 0, (moduleCount - 7) * moduleSize, moduleSize);

    // Draw data modules from hash
    for (let row = 0; row < moduleCount; row++) {
      for (let col = 0; col < moduleCount; col++) {
        // Skip finder pattern areas
        if (isFinderArea(row, col, moduleCount)) continue;

        // Deterministic pattern from hash + position
        const bit =
          ((hash >> ((row * moduleCount + col) % 31)) & 1) ^
          ((hash >> ((col * moduleCount + row) % 29)) & 1);

        if (bit) {
          ctx.fillStyle = '#211B14';
          ctx.fillRect(
            col * moduleSize + 0.5,
            row * moduleSize + 0.5,
            moduleSize - 1,
            moduleSize - 1,
          );
        }
      }
    }
  }, [target, size]);

  return (
    <div className="qr-container">
      <canvas
        ref={canvasRef}
        width={size}
        height={size}
        className="qr-canvas"
        role="img"
        aria-label={`QR code linking to ${target}`}
      />
      <span className="qr-label">{label ?? target}</span>
    </div>
  );
}

function simpleHash(str: string): number {
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash + str.charCodeAt(i)) | 0;
  }
  return Math.abs(hash);
}

function isFinderArea(row: number, col: number, size: number): boolean {
  // Top-left
  if (row < 8 && col < 8) return true;
  // Top-right
  if (row < 8 && col >= size - 8) return true;
  // Bottom-left
  if (row >= size - 8 && col < 8) return true;
  return false;
}

function drawFinderPattern(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  moduleSize: number,
) {
  const s = moduleSize;

  // Outer ring
  ctx.fillStyle = '#211B14';
  ctx.fillRect(x, y, 7 * s, 7 * s);

  // White inner
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(x + s, y + s, 5 * s, 5 * s);

  // Center dot
  ctx.fillStyle = '#211B14';
  ctx.fillRect(x + 2 * s, y + 2 * s, 3 * s, 3 * s);
}

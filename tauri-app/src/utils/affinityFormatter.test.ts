import { formatAffinityMode } from './affinityFormatter';

describe('formatAffinityMode', () => {
  it('returns emerald badge for ExcludeFromCapture mode', () => {
    const info = formatAffinityMode('ExcludeFromCapture');
    expect(info.label).toBe('Fully Masked');
    expect(info.badgeClass).toBe('badge-emerald');
  });

  it('returns amber badge for Monitor mode', () => {
    const info = formatAffinityMode('Monitor');
    expect(info.label).toBe('Monitor Only');
    expect(info.badgeClass).toBe('badge-amber');
  });

  it('returns slate badge for None mode', () => {
    const info = formatAffinityMode('None');
    expect(info.label).toBe('Unshielded');
    expect(info.badgeClass).toBe('badge-slate');
  });
});

import type { LiftType, SexValue } from './types';

// Batch DOTS calculation for improved performance
function batchCalculateDOTS(lifts: Array<{ lift: number; bw: number; sex: SexValue }>): number[] {
  if (!window.calculate_dots_with_gender_wasm) {
    // Fallback to individual calculations
    return lifts.map(({ lift, bw, sex }) => calculateDOTS(lift, bw, sex));
  }

  // Use WASM for batch processing
  return lifts.map(({ lift, bw, sex }) => {
    const isMale = sex === 'M' || sex === 'Male';
    return window.calculate_dots_with_gender_wasm!(lift, bw, isMale);
  });
}

// Helper function to calculate gender-specific DOTS score using WASM
function calculateDOTS(liftKg: number, bodyweightKg: number, sex: SexValue | null = null): number {
  const sexValue: SexValue = sex || window.currentSex;
  const isMale = sexValue === 'M' || sexValue === 'Male';

  if (window.calculate_dots_with_gender_wasm) {
    return window.calculate_dots_with_gender_wasm(liftKg, bodyweightKg, isMale);
  }

  // Fallback JavaScript implementation with gender-specific coefficients
  let a: number;
  let b: number;
  let c: number;
  let d: number;
  let e: number;

  if (isMale) {
    // Male coefficients
    a = -307.75076;
    b = 24.0900756;
    c = -0.1918759221;
    d = 0.0007391293;
    e = -0.000001093;
  } else {
    // Female coefficients
    a = -57.96288;
    b = 13.6175032;
    c = -0.1126655495;
    d = 0.0005158568;
    e = -0.0000010706;
  }

  const denominator =
    a + b * bodyweightKg + c * Math.pow(bodyweightKg, 2) + d * Math.pow(bodyweightKg, 3) + e * Math.pow(bodyweightKg, 4);

  return liftKg * 500.0 / denominator;
}

// Fallback strength level calculation
function getStrengthLevel(dotsScore: number, isMale: boolean = true): string {
  return getStrengthLevelForLift(dotsScore, 'total', isMale);
}

type StrengthThresholds = {
  beginner: number;
  novice: number;
  intermediate: number;
  advanced: number;
  elite: number;
  worldClass: number;
};

type LiftStandards = Record<'male' | 'female', StrengthThresholds>;

// Gender and lift-specific strength level calculation
function getStrengthLevelForLift(dotsScore: number, liftType: LiftType | 'total', isMale: boolean = true): string {
  // Realistic DOTS-based strength standards per single lift
  const standards: Record<LiftType | 'total', LiftStandards> = {
    squat: {
      male: { beginner: 61, novice: 102, intermediate: 132, advanced: 163, elite: 187, worldClass: Infinity },
      female: { beginner: 58, novice: 96, intermediate: 125, advanced: 154, elite: 177, worldClass: Infinity }
    },
    bench: {
      male: { beginner: 41, novice: 69, intermediate: 89, advanced: 110, elite: 127, worldClass: Infinity },
      female: { beginner: 39, novice: 65, intermediate: 85, advanced: 104, elite: 120, worldClass: Infinity }
    },
    deadlift: {
      male: { beginner: 63, novice: 105, intermediate: 136, advanced: 167, elite: 192, worldClass: Infinity },
      female: { beginner: 59, novice: 99, intermediate: 128, advanced: 158, elite: 182, worldClass: Infinity }
    },
    total: {
      male: { beginner: 200, novice: 300, intermediate: 400, advanced: 500, elite: 600, worldClass: 700 },
      female: { beginner: 180, novice: 270, intermediate: 360, advanced: 450, elite: 540, worldClass: 630 }
    }
  };

  const genderKey = isMale ? 'male' : 'female';
  const liftStandards = standards[liftType] || standards.total;
  const thresholds = liftStandards[genderKey];

  if (dotsScore < thresholds.beginner) return 'Beginner';
  if (dotsScore < thresholds.novice) return 'Novice';
  if (dotsScore < thresholds.intermediate) return 'Intermediate';
  if (dotsScore < thresholds.advanced) return 'Advanced';
  if (dotsScore < thresholds.elite) return 'Elite';
  return 'World Class';
}

// Fallback Wilks 2020 calculation
function calculateWilks(liftKg: number, bodyweightKg: number, isMale: boolean): number {
  const bw = bodyweightKg;

  if (isMale) {
    // Male Wilks 2020 coefficients
    const a = 47.46178854;
    const b = 8.472061379;
    const c = 0.07369410346;
    const d = -0.001395833811;
    const e = 7.07665973070743e-06;
    const f = -1.20804336482315e-08;
    const denominator = a + b * bw + c * bw * bw + d * bw * bw * bw + e * bw * bw * bw * bw + f * bw * bw * bw * bw * bw;
    return liftKg * 600.0 / denominator;
  }

  // Female Wilks 2020 coefficients
  const a = -125.4255398;
  const b = 13.71219419;
  const c = -0.03307250631;
  const d = -0.001050400051;
  const e = 9.38773881462799e-06;
  const f = -2.3334613884954e-08;
  const denominator = a + b * bw + c * bw * bw + d * bw * bw * bw + e * bw * bw * bw * bw + f * bw * bw * bw * bw * bw;
  return liftKg * 600.0 / denominator;
}

// Fallback IPF GL Points calculation
function calculateIPFGLPoints(liftKg: number, bodyweightKg: number, isMale: boolean): number {
  if (isMale) {
    return 1199.72839 / (1025.18162 - 0.00921 * bodyweightKg) * liftKg;
  }
  return 610.32796 / (1045.59282 - 0.03048 * bodyweightKg) * liftKg;
}

// Fallback strength level color
function getStrengthLevelColor(level: string): string {
  switch (level) {
    case 'Untrained':
      return '#495057';
    case 'Beginner':
      return '#6c757d';
    case 'Novice':
      return '#28a745';
    case 'Intermediate':
      return '#17a2b8';
    case 'Advanced':
      return '#ffc107';
    case 'Elite':
      return '#fd7e14';
    case 'World Class':
      return '#dc3545';
    default:
      return '#6c757d';
  }
}

// Function to update user metrics display for modern UI
function updateUserMetrics(bodyweight: number | null | undefined, userLift: number | null | undefined): void {
  console.log('updateUserMetrics called with:', {
    bodyweight,
    userLift,
    isValid: !!(bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift))
  });

  if (bodyweight && userLift && !isNaN(bodyweight) && !isNaN(userLift)) {
    try {
      let dots: number;
      let wilks: number;
      let ipfGLPoints: number;
      let level: string;
      let color: string;
      const isMale = window.currentSex === 'M';

      console.log('Calculating metrics with:', { bodyweight, userLift, isMale });

      if (
        window.calculate_dots_and_level_for_lift_with_gender_wasm &&
        window.calculate_wilks_wasm &&
        window.calculate_ipf_gl_points_wasm
      ) {
        console.log('Using WASM calculations with lift type:', window.currentLiftType);

        const dotsResult = window.calculate_dots_and_level_for_lift_with_gender_wasm(
          userLift,
          bodyweight,
          isMale,
          window.currentLiftType
        );
        console.log('WASM DOTS result:', dotsResult);

        dots = dotsResult.dots;
        level = dotsResult.level;
        color = dotsResult.color;
        wilks = window.calculate_wilks_wasm(userLift, bodyweight, isMale);
        ipfGLPoints = window.calculate_ipf_gl_points_wasm(userLift, bodyweight, isMale);
      } else {
        console.log('Using fallback calculations');
        // Fallback calculation
        dots = calculateDOTS(userLift, bodyweight);
        wilks = calculateWilks(userLift, bodyweight, isMale);
        ipfGLPoints = calculateIPFGLPoints(userLift, bodyweight, isMale);
        level = getStrengthLevelForLift(dots, window.currentLiftType, isMale);
        color = getStrengthLevelColor(level);
      }

      console.log('Calculated values:', { dots, wilks, ipfGLPoints, level, color });

      // Update strength level display
      const strengthLevelElement = document.getElementById('strengthLevel');
      if (strengthLevelElement) {
        strengthLevelElement.innerHTML = '<div class="strength-badge" style="background-color: ' + color + ';">ĐY?<‹÷? ' + level + '</div>';
      }

      // Update all scoring metrics
      const userDotsEl = document.getElementById('userDotsScore');
      const userWilksEl = document.getElementById('userWilks');
      const userIPFGLPointsEl = document.getElementById('userIPFGLPoints');

      if (userDotsEl) userDotsEl.textContent = dots.toFixed(1);
      if (userWilksEl) userWilksEl.textContent = wilks.toFixed(1);
      if (userIPFGLPointsEl) userIPFGLPointsEl.textContent = ipfGLPoints.toFixed(1);

      // Performance card is always visible - no need to show/hide
    } catch (error) {
      console.error('Error calculating user metrics:', error);
    }
  } else {
    console.log('Invalid data for user metrics');
  }
}

// Expose functions to global scope
window.calculateDOTS = calculateDOTS;
window.batchCalculateDOTS = batchCalculateDOTS;
window.getStrengthLevel = getStrengthLevel;
window.getStrengthLevelForLift = getStrengthLevelForLift;
window.calculateWilks = calculateWilks;
window.calculateIPFGLPoints = calculateIPFGLPoints;
window.getStrengthLevelColor = getStrengthLevelColor;
window.updateUserMetrics = updateUserMetrics;

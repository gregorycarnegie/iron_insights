mod bodyfat;
mod men_vs_women;
mod nerds;
mod one_rm;
mod plate_calc;
mod ranking;
mod shared;

pub(super) use bodyfat::BodyfatPage;
pub(super) use men_vs_women::{MenVsWomenCtx, MenVsWomenPage};
pub(super) use nerds::{NerdsCtx, NerdsPage};
pub(super) use one_rm::OneRmPage;
pub(super) use plate_calc::PlateCalcPage;
pub(super) use ranking::{RankingCtx, RankingPage};

use crate::core::{HeatmapBin, HistogramBin};
use crate::webapp::models::{CrossSexComparison, SliceSummary, TrendSeries};
use leptos::html::Canvas;
use leptos::prelude::*;

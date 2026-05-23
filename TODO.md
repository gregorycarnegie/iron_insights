# Iron Insights TODO

Current focus: website + published dataset only. The Android client has been removed from this repo for now so product and engineering effort stays centered on the site.

## Website

- [ ] Add integration or snapshot coverage for the highest-risk flows in `iron_insights_web`
- [ ] Break up the largest UI modules (`trends.rs`, `plate_calc.rs`, `one_rm.rs`) before they sprawl further
- [ ] Keep tightening the first-run and mobile experience in the main site
- [ ] Continue burning down the detailed UI checklist in `iron_insights_web/todo.md`

## Pipeline And Publishing

- [ ] Revisit content-addressed `.bin` filenames if hosting moves from GitHub Pages to a CDN with immutable cache headers
- [ ] Keep the published-data contract in the root `README.md` and `iron_insights_web/README.md` synchronized
- [ ] Track payload size, slice counts, and refresh safety rails as the dataset grows

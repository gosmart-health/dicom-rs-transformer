# Development Workflow & Branching Strategy

This document outlines the workflow, branching standards, and release processes for the `dicom-rs-transformer` repository.

---

## 1. Branch Hierarchy

We follow a structured branching model to ensure development stability.

### `dev` (Latest Pre-Release Candidate)
- **Status**: The latest integration point for upcoming releases.
- **Rule**: `dev` is considered the most stable pre-release branch. All developers should branch off `dev` for feature work and bug fixes, and all feature pull requests must target the `dev` branch.

### `main` (Production & Release)
- **Status**: Production-ready code reflecting official releases.
- **Rule**: Merges to `main` are reserved for release tags and final deployments.

---

## 2. GitHub Issues & Branching Strategy

We use GitHub Issues extensively to track and organize work.

### Issue Naming Convention
Issue headings must start with one of the following prefix keywords:
- `feat`: New features or capabilities.
- `fix`: Bug fixes.
- `perf`: Performance improvements.
- `chore`: Maintenance, configuration, documentation, or dependency updates.

### Branch Creation & Naming Rules
When addressing a GitHub Issue:
- **Branch Creation**: Use GitHub's branch creation feature directly from the issue page when possible.
- **Branch Name Format**: The branch must start with your initials, followed by a slash (`/`), and include the ticket number.
  - *Format*: `<initials>/<ticket-number>-<short-description>`
  - *Example*: `mt/123-feat-sequence-dsl`

### Managing Large Features
For larger features encompassing multiple smaller steps:
- Create a **top-level issue** and branch off it using the format above.
- Create **sub-issues** for the individual components or steps.
- Work and commit directly to the top-level feature branch—**there is no need to branch for every sub-issue**.
- Reference the sub-issue ticket numbers in your commit messages (e.g. `feat: implement validation schema (#125)`).

---

## 3. Feature Development Loop

When developing new features or fixing bugs:

1. **Branch off `dev`**: Create your branch from the current `dev` branch.
   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b <initials>/<ticket-number>-<short-description>
   ```
2. **Submit Pull Request**: Open a pull request targeting the `dev` branch.
3. **CI Testing**: GitHub Actions compiles and runs automated tests to ensure correctness for all pull requests before merging.

---

## 4. QA & Staging Process

Before code from `dev` is promoted to `main`, it goes through a QA verification process:

1. **Staging / Release Branching**: A dedicated staging/release branch (e.g., `release/vX.Y.Z` or a temporary staging branch) may be spawned from `dev` to isolate changes for testing.
2. **QA Validation**: Thorough integration testing, regression tests, and user acceptance testing (UAT) are performed on this staging candidate.
3. **Promotion**: Once validated, the staging candidate is merged into `main` and tagged for release.

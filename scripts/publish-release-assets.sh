#!/bin/sh
set -eu

repository=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}
tag=${GITHUB_REF_NAME:?GITHUB_REF_NAME is required}
pre_build_metadata=${tag%%+*}

case "$pre_build_metadata" in
  *-*)
    if ! published=$(gh release view "$tag" --repo "$repository" --json isPrerelease,isDraft --jq '.isPrerelease == true and .isDraft == false'); then
      printf '%s\n' "soulmate: preview release lookup failed" >&2
      exit 1
    fi
    if test "$published" != true; then
      printf '%s\n' "soulmate: preview release must be a published prerelease" >&2
      exit 1
    fi
    ;;
  *)
    if ! gh release view "$tag" --repo "$repository" >/dev/null 2>&1; then
      gh release create "$tag" --repo "$repository" --verify-tag --title "Soulmate $tag" --generate-notes
    fi
    ;;
esac

gh release upload "$tag" --repo "$repository" dist/*

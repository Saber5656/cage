# frozen_string_literal: true

require "minitest/autorun"
require "fileutils"
require "tmpdir"
require_relative "check-workflow-triggers"

class WorkflowTriggerGuardTest < Minitest::Test
  def test_accepts_push_and_pull_request_triggers
    yaml = <<~YAML
      name: CI
      on:
        push:
        pull_request:
    YAML

    assert_nil WorkflowTriggerGuard.trigger_policy_error(yaml)
  end

  def test_rejects_scheduled_trigger
    yaml = <<~YAML
      name: Security
      on:
        schedule:
          - cron: "0 0 * * 0"
    YAML

    assert_match(/schedule/, WorkflowTriggerGuard.trigger_policy_error(yaml))
  end

  def test_rejects_inline_scheduled_trigger
    error = WorkflowTriggerGuard.trigger_policy_error("on: { schedule: [{ cron: '0 0 * * 0' }] }\n")

    assert_match(/schedule/, error)
  end

  def test_rejects_workflow_dispatch
    error = WorkflowTriggerGuard.trigger_policy_error("on: [push, workflow_dispatch]\n")

    assert_match(/workflow_dispatch/, error)
  end

  def test_rejects_additional_mapping_trigger
    yaml = <<~YAML
      on:
        push:
        issues:
          types: [opened]
    YAML

    assert_match(/issues/, WorkflowTriggerGuard.trigger_policy_error(yaml))
  end

  def test_rejects_alias_under_on
    yaml = <<~YAML
      triggers: &triggers
        schedule:
          - cron: "0 0 * * 0"
      on: *triggers
    YAML

    assert_match(/aliases/, WorkflowTriggerGuard.trigger_policy_error(yaml))
  end

  def test_rejects_duplicate_on_that_hides_unsupported_trigger
    yaml = <<~YAML
      on: [push, pull_request]
      on: [workflow_dispatch]
    YAML

    error = WorkflowTriggerGuard.trigger_policy_error(yaml)

    assert_match(/duplicate top-level mapping key.*on/, error)
  end

  def test_rejects_other_duplicate_top_level_keys
    yaml = <<~YAML
      name: first
      name: second
      on: push
    YAML

    assert_match(/duplicate top-level mapping key.*name/, WorkflowTriggerGuard.trigger_policy_error(yaml))
  end

  def test_accepts_allowed_sequence_triggers
    assert_nil WorkflowTriggerGuard.trigger_policy_error("on: [push, pull_request]\n")
  end

  def test_ignores_trigger_words_outside_the_trigger_mapping
    yaml = <<~YAML
      name: CI
      on: [push, pull_request]
      jobs:
        audit:
          steps:
            - run: echo schedule
    YAML

    assert_nil WorkflowTriggerGuard.trigger_policy_error(yaml)
  end

  def test_validate_rejects_invalid_yaml
    Dir.mktmpdir do |root|
      workflow_directory = File.join(root, ".github", "workflows")
      FileUtils.mkdir_p(workflow_directory)
      File.write(File.join(workflow_directory, "invalid.yml"), "on:\n  push: [\n")

      refute WorkflowTriggerGuard.validate(root)
    end
  end
end

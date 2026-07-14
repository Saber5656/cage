#!/usr/bin/env ruby
# frozen_string_literal: true

require "pathname"
require "psych"

module WorkflowTriggerGuard
  ALLOWED_TRIGGERS = %w[push pull_request].freeze

  module_function

  def mapping_value(node, key)
    return unless node.is_a?(Psych::Nodes::Mapping)

    node.children.each_slice(2) do |key_node, value_node|
      return value_node if key_node.is_a?(Psych::Nodes::Scalar) && key_node.value == key
    end

    nil
  end

  def duplicate_mapping_keys(node)
    return unless node.is_a?(Psych::Nodes::Mapping)

    keys = node.children.each_slice(2).map do |key_node, _value_node|
      return unless key_node.is_a?(Psych::Nodes::Scalar)

      key_node.value
    end

    keys.group_by(&:itself).select { |_key, values| values.length > 1 }.keys.sort
  end

  def contains_alias?(node)
    return true if node.is_a?(Psych::Nodes::Alias)
    children = node.children if node.respond_to?(:children)
    return false if children.nil?

    children.any? { |child| contains_alias?(child) }
  end

  def trigger_names(node)
    case node
    when Psych::Nodes::Mapping
      node.children.each_slice(2).map do |key_node, _value_node|
        return unless key_node.is_a?(Psych::Nodes::Scalar)

        key_node.value
      end
    when Psych::Nodes::Sequence
      node.children.map do |child|
        return unless child.is_a?(Psych::Nodes::Scalar)

        child.value
      end
    when Psych::Nodes::Scalar
      [node.value]
    end
  end

  def trigger_policy_error(yaml)
    document = Psych.parse(yaml)
    root = document&.root
    return "workflow root must be a mapping" unless root.is_a?(Psych::Nodes::Mapping)

    duplicate_keys = duplicate_mapping_keys(root)
    return "top-level mapping keys must be scalars" if duplicate_keys.nil?
    unless duplicate_keys.empty?
      return "duplicate top-level mapping key(s): #{duplicate_keys.join(', ')}"
    end

    triggers = mapping_value(root, "on")
    return "workflow must define an on trigger" if triggers.nil?
    return "YAML aliases are not allowed under on" if contains_alias?(triggers)

    names = trigger_names(triggers)
    return "on must contain scalar trigger names" if names.nil?

    unsupported = names.uniq - ALLOWED_TRIGGERS
    return if unsupported.empty?

    "unsupported workflow trigger(s): #{unsupported.sort.join(', ')}"
  end

  def workflow_files(root)
    directory = Pathname(root).join(".github", "workflows")
    Dir[directory.join("*.{yml,yaml}").to_s].sort
  end

  def validate(root)
    failures = []

    workflow_files(root).each do |path|
      begin
        if (policy_error = trigger_policy_error(File.read(path)))
          failures << "#{path}: #{policy_error}; only push and pull_request are allowed for pre-alpha"
        end
      rescue Psych::SyntaxError => error
        failures << "#{path}: invalid workflow YAML: #{error.problem} at line #{error.line}"
      end
    end

    failures.each { |failure| warn failure }
    failures.empty?
  end
end

if $PROGRAM_NAME == __FILE__
  root = ARGV.fetch(0, File.expand_path("..", __dir__))
  exit(WorkflowTriggerGuard.validate(root) ? 0 : 1)
end

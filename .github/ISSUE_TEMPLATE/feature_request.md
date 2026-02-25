name: "Feature Request"
description: "Suggest a new feature or improvement"
labels: ["enhancement"]
body:
  - type: textarea
    id: feature-description
    attributes:
      label: Feature Description
      description: A clear and concise description of what you want to happen.
    validations:
      required: true
  - type: textarea
    id: forensic-value
    attributes:
      label: Forensic/Operational Value
      description: Why is this feature useful for Law Enforcement or Incident Response?
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
      description: Have you tried other tools that do this?

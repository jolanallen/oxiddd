name: "Bug Report"
description: "Report a software bug or forensic inconsistency"
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to fill out this bug report! 
        Reliability is critical for `oxiddd`.
  - type: textarea
    id: description
    attributes:
      label: Description
      description: What happened?
      placeholder: "e.g., Acquisition failed at block X, or Hash mismatch on re-read."
    validations:
      required: true
  - type: input
    id: version
    attributes:
      label: oxiddd version
      description: Output of `oxiddd --version`
    validations:
      required: true
  - type: input
    id: command
    attributes:
      label: Command used
      description: The exact command you ran
      placeholder: "oxiddd if=/dev/sdb of=image.dd ..."
    validations:
      required: true
  - type: textarea
    id: environment
    attributes:
      label: Environment context
      description: OS version, Kernel version, Disk type (SSD/HDD/USB), etc.
  - type: textarea
    id: logs
    attributes:
      label: Logs / Console Output
      description: Please paste any error messages or logs here.
      render: shell

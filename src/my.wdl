version 1.0

task foo {
    input {
        String name
        Int x = 5
    }

    command <<<
        echo hello ~{name}
        echo x is ~{x}
    >>>

    output {
        File stdout = stdout()
    }
}

workflow exampleWorkflow {
    input {
        String wf_name
        Int wf_x
    }

    call foo {
        input:
            name = wf_name,
            x = wf_x
    }

    output {
        File foo_stdout = foo.stdout
    }
}


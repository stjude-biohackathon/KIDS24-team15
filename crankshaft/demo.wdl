version 1.2

task samtools_flagstat {
    input {
        String url
        Int count
    }

    command <<<
        samtools flagstat <(wget -O - -q '~{url}' | samtools view -h | head -n ~{count})
    >>>

    requirements {
        container: "quay.io/biocontainers/samtools:1.19.2--h50ea8bc_0"
    }

    output {
        String stats = read_string(stdout())
    }
}

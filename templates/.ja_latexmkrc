#!/usr/bin/env perl

# LaTeX
$latex = 'uplatex -interaction=nonstopmode -file-line-error -halt-on-error %O %S';
$max_repeat = 5;

# # BibTeX
$bibtex = 'upbibtex %O %S';
$biber = 'biber --bblencoding=utf8 -u -U --output_safechars %O %S';

# DVI / PDF
# dvi -> pdf の変換に使用するものとしてdvipdfmxを指定
$dvipdf = 'dvipdfmx %O -o %D %S'; # dvipdfmx [options] -o output.pdf input.dvi ということになる
$pdf_mode = 3; # dviファイルからpdfを作成する場合 uplatexならこれ

# output directory 
# この.latexmkrcと同じディレクトリにあるoutディレクトリに出力する
{
    use Cwd;
    my $current_dir_path = Cwd::getcwd();
    print "current directory path: $current_dir_path\n";

    my $rc_file_path = "$current_dir_path/.latexmkrc"; # この.latexmkrcファイルの絶対パス。もしこのファイルと同じディレクトリでlatexmkが実行されたならこれでok

    # -r オプションでこの.latexmkrcファイルを指定していた場合はこっち
    for my $arg ( @ARGV ){
        if( $arg =~ /\.latexmkrc$/ ){
            $rc_file_path = ($arg=~m!^/!)? $arg : "$current_dir_path/$arg";
        }
    }
    print "latexmkrc file path: $rc_file_path\n";

    my $project_dir_path = $rc_file_path =~ s/\/\.latexmkrc$//r;

    $out_dir = "$project_dir_path/out"; # e.g. /Users/username/project1/out
    print "output directory path: $out_dir\n";
}
